use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use rayon::prelude::*;
use crate::parser::parse_rst_multiple;
use crate::aggregator::DirectiveWithSource; // DirectiveWithSource now has an `id` field
use crate::extractor::RstExtractor;
use std::sync::{Arc, Mutex}; // For watch mode return types
use std::collections::HashMap; // For process_files_watch return type

/// A struct to process RST files and find directives
pub struct Processor {
    target_directives: Vec<String>,
}

impl Processor {
    pub fn new(target_directives: Vec<String>) -> Self {
        Processor { target_directives }
    }

    /// Process a single file, canonicalize its path, generate directive IDs, and find directives.
    pub fn process_file<P: AsRef<Path>>(&self, file_path_ref: P) -> Result<Vec<DirectiveWithSource>, Box<dyn Error>> {
        let original_path = file_path_ref.as_ref();
        let canonical_file_path = match fs::canonicalize(original_path) {
            Ok(p) => p,
            Err(e) => {
                // If canonicalization fails (e.g. file deleted during watch), return error or empty.
                // For a direct call, failing might be better.
                return Err(Box::new(std::io::Error::new(
                    e.kind(),
                    format!("Failed to canonicalize path {}: {}", original_path.display(), e)
                )));
            }
        };
        let canonical_source_file_str = canonical_file_path.to_string_lossy().to_string();

        let content = fs::read_to_string(&canonical_file_path)?;
        let rst_content = RstExtractor::extract_from_file(&canonical_file_path, &content);
        
        let target_directives_refs: Vec<&str> = self.target_directives.iter().map(|s| s.as_str()).collect();
        let directives_with_lines = parse_rst_multiple(&rst_content, &target_directives_refs);
        
        let directives_with_source = directives_with_lines.into_iter().map(|(directive, line_number)| { // Removed mut from directive
            // Generate ID: use :id: option if present, otherwise fallback
            let id = directive.options.get("id")
                .map(|id_val| id_val.trim().to_string())
                .filter(|id_val| !id_val.is_empty())
                .unwrap_or_else(|| {
                    format!("{}:{}:{}",
                        canonical_source_file_str, // Use canonical path string for ID
                        directive.name,
                        line_number // line_number from parse_rst_multiple is usize
                    )
                });
            
            // Ensure the :id: option is stored if it was used for the ID
            if !directive.options.contains_key("id") && id.starts_with(&canonical_source_file_str) == false { // Heuristic: if id is not path-based, it was from :id:
                 if let Some(opt_id) = directive.options.get("id") {
                    if opt_id.trim() == id {
                        // ID came from option, ensure it's stored as such if not already.
                        // This logic might be redundant if parse_directive_body preserves options correctly.
                    }
                 } else {
                     // If ID was generated not from an option, but we want to store the generated ID as an option.
                     // This might be controversial. For now, let's assume ID is for internal tracking.
                     // If :id: was present, it's used. If not, a unique one is generated.
                     // The `id` field in `DirectiveWithSource` stores this unique ID.
                 }
            }


            DirectiveWithSource {
                directive,
                source_file: canonical_source_file_str.clone(),
                line_number: Some(line_number), // line_number from parse_rst_multiple is usize, wrap in Some()
                id, // Populate the new id field
            }
        }).collect();
        
        Ok(directives_with_source)
    }

    /// Process multiple files in parallel (for non-watch mode).
    /// Returns a flat Vec of all found directives with populated IDs and canonical source_file.
    pub fn process_files(&self, file_paths: Vec<PathBuf>) -> Result<Vec<DirectiveWithSource>, Box<dyn Error + Send + Sync>> {
        let results: Vec<Result<Vec<DirectiveWithSource>, String>> = file_paths.par_iter()
            .map(|file_path| {
                self.process_file(file_path)
                    .map_err(|e| e.to_string()) // Convert error to String
            })
            .collect();
        
        let mut all_directives = Vec::new();
        let mut errors_accumulator: Vec<String> = Vec::new();
        
        for result in results {
            match result {
                Ok(directives) => all_directives.extend(directives),
                Err(e_str) => errors_accumulator.push(e_str),
            }
        }
        
        if !errors_accumulator.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Errors occurred while processing files: {}", errors_accumulator.join("\n"))
            )));
        }
        Ok(all_directives)
    }

    /// Process a single file for watch mode, returning Vec<Arc<Mutex<DirectiveWithSource>>>.
    /// Handles ID generation and path canonicalization.
    pub fn process_file_watch<P: AsRef<Path>>(&self, file_path_ref: P) -> Result<Vec<Arc<Mutex<DirectiveWithSource>>>, Box<dyn Error>> {
        let directives = self.process_file(file_path_ref)?; // Reuses the updated process_file
        Ok(directives.into_iter().map(|dws| Arc::new(Mutex::new(dws))).collect())
    }

    /// Process multiple files for watch mode initial scan.
    /// Returns a map of canonical_path -> Vec<Arc<Mutex<DirectiveWithSource>>>.
    pub fn process_files_watch(&self, file_paths: Vec<PathBuf>) -> Result<HashMap<PathBuf, Vec<Arc<Mutex<DirectiveWithSource>>>>, Box<dyn Error + Send + Sync>> {
        let results: Vec<Result<(PathBuf, Vec<Arc<Mutex<DirectiveWithSource>>>), String>> = file_paths.par_iter()
            .map(|file_path_orig| {
                let canonical_file_path = match fs::canonicalize(file_path_orig) {
                     Ok(p) => p,
                     Err(e) => return Err(format!("Failed to canonicalize path {}: {}", file_path_orig.display(), e)),
                };
                match self.process_file_watch(&canonical_file_path) {
                    Ok(arc_directives) => Ok((canonical_file_path, arc_directives)),
                    Err(e) => Err(format!("Error processing file {}: {}", canonical_file_path.display(), e)),
                }
            })
            .collect();

        let mut processed_map: HashMap<PathBuf, Vec<Arc<Mutex<DirectiveWithSource>>>> = HashMap::new();
        let mut errors_accumulator: Vec<String> = Vec::new();

        for result in results {
            match result {
                Ok((path, directives)) => {
                    processed_map.insert(path, directives);
                }
                Err(e_str) => errors_accumulator.push(e_str),
            }
        }

        if !errors_accumulator.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Errors occurred during initial watch scan: {}", errors_accumulator.join("\n"))
            )));
        }
        Ok(processed_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_process_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rst");
        
        let rst_content = r#"
.. directive1::
   :option1: value1
   :id: custom-id-1

   Content for directive1.

.. directive2::
   :option2: value2

   Content for directive2.

.. directive1::
   :option3: value3

   More content for directive1.
"#;
        
        File::create(&file_path).unwrap().write_all(rst_content.as_bytes()).unwrap();
        let canonical_path_str = fs::canonicalize(&file_path).unwrap().to_string_lossy().to_string();
        
        let processor = Processor::new(vec!["directive1".to_string(), "directive2".to_string()]);
        let result = processor.process_file(&file_path).unwrap();
        
        assert_eq!(result.len(), 3);
        
        assert_eq!(result[0].directive.name, "directive1");
        assert_eq!(result[0].id, "custom-id-1"); // Uses :id: option
        assert_eq!(result[0].source_file, canonical_path_str);
        assert!(result[0].line_number.is_some());

        assert_eq!(result[1].directive.name, "directive2");
        let expected_id2 = format!("{}:{}:{}", canonical_path_str, "directive2", result[1].line_number.unwrap_or(0));
        assert_eq!(result[1].id, expected_id2); // Generated ID
        assert_eq!(result[1].source_file, canonical_path_str);

        assert_eq!(result[2].directive.name, "directive1");
        let expected_id3 = format!("{}:{}:{}", canonical_path_str, "directive1", result[2].line_number.unwrap_or(0));
        assert_eq!(result[2].id, expected_id3); // Generated ID
    }

    #[test]
    fn test_process_files() {
        let temp_dir = tempdir().unwrap();
        let file1_path = temp_dir.path().join("file1.rst");
        let file2_path = temp_dir.path().join("file2.rst");
        
        let rst_content1 = r#".
.. directive1::
   :id: d1f1

   Content for directive1 in file1."#;
        
        let rst_content2 = r#"
.. directive2::
   :id: d2f2

   Content for directive2 in file2.

.. directive1::
   :id: d1f2

   Content for directive1 in file2."#;
        
        File::create(&file1_path).unwrap().write_all(rst_content1.as_bytes()).unwrap();
        File::create(&file2_path).unwrap().write_all(rst_content2.as_bytes()).unwrap();
        
        let processor = Processor::new(vec!["directive1".to_string(), "directive2".to_string()]);
        let result_vec = processor.process_files(vec![file1_path.clone(), file2_path.clone()]).unwrap();
        
        assert_eq!(result_vec.len(), 3);
        
        let d1f1_opt = result_vec.iter().find(|d| d.id == "d1f1");
        assert!(d1f1_opt.is_some());
        assert_eq!(d1f1_opt.unwrap().source_file, fs::canonicalize(&file1_path).unwrap().to_string_lossy());

        let d2f2_opt = result_vec.iter().find(|d| d.id == "d2f2");
        assert!(d2f2_opt.is_some());
        assert_eq!(d2f2_opt.unwrap().source_file, fs::canonicalize(&file2_path).unwrap().to_string_lossy());

        let d1f2_opt = result_vec.iter().find(|d| d.id == "d1f2");
        assert!(d1f2_opt.is_some());
        assert_eq!(d1f2_opt.unwrap().source_file, fs::canonicalize(&file2_path).unwrap().to_string_lossy());
    }
}

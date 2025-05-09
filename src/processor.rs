use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use rayon::prelude::*;
use crate::parser::parse_rst_multiple;
use crate::aggregator::DirectiveWithSource;
use crate::extractor::RstExtractor;

/// A struct to process RST files and find directives
pub struct Processor {
    target_directives: Vec<String>,
}

impl Processor {
    /// Create a new Processor with the specified target directives
    pub fn new(target_directives: Vec<String>) -> Self {
        Processor {
            target_directives,
        }
    }

    /// Process a single file and find directives
    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<DirectiveWithSource>, Box<dyn Error>> {
        let file_path = file_path.as_ref();
        let content = fs::read_to_string(file_path)?;
        
        // Extract RST content based on file extension
        let rst_content = RstExtractor::extract_from_file(file_path, &content);
        
        // Convert target_directives from Vec<String> to Vec<&str> for parse_rst_multiple
        let target_directives_refs: Vec<&str> = self.target_directives.iter().map(|s| s.as_str()).collect();
        
        // Parse the file content to find directives with line numbers
        let directives_with_lines = parse_rst_multiple(&rst_content, &target_directives_refs);
        
        // Convert to DirectiveWithSource
        let source_file = file_path.to_string_lossy().to_string();
        let directives_with_source = directives_with_lines.into_iter().map(|(directive, line_number)| {
            DirectiveWithSource {
                directive,
                source_file: source_file.clone(),
                line_number: Some(line_number), // Use the line number from the parser
            }
        }).collect();
        
        Ok(directives_with_source)
    }

    /// Process multiple files in parallel and find directives
    pub fn process_files(&self, file_paths: Vec<PathBuf>) -> Result<Vec<DirectiveWithSource>, Box<dyn Error + Send + Sync>> {
        // Process files in parallel using rayon
        let results: Vec<Result<Vec<DirectiveWithSource>, _>> = file_paths.par_iter()
            .map(|file_path| {
                self.process_file(file_path)
                    .map_err(|e| format!("Error processing file {}: {}", file_path.display(), e))
                    .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) })
            })
            .collect();
        
        // Collect all successful results and report any errors
        let mut all_directives = Vec::new();
        let mut errors = Vec::new();
        
        for result in results {
            match result {
                Ok(directives) => all_directives.extend(directives),
                Err(e) => errors.push(e.to_string()),
            }
        }
        
        // If there were any errors, report them
        if !errors.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Errors occurred while processing files: {}", errors.join(", "))
            )));
        }
        
        Ok(all_directives)
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
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rst");
        
        // Create a test RST file
        let rst_content = r#"
.. directive1::
   :option1: value1

   Content for directive1.

.. directive2::
   :option2: value2

   Content for directive2.

.. directive1::
   :option3: value3

   More content for directive1.
"#;
        
        File::create(&file_path).unwrap().write_all(rst_content.as_bytes()).unwrap();
        
        // Create processor to find directive1 and directive2
        let processor = Processor::new(vec!["directive1".to_string(), "directive2".to_string()]);
        let result = processor.process_file(&file_path).unwrap();
        
        // Should find 3 directives
        assert_eq!(result.len(), 3);
        
        // Check directive names
        assert_eq!(result[0].directive.name, "directive1");
        assert_eq!(result[1].directive.name, "directive2");
        assert_eq!(result[2].directive.name, "directive1");
        
        // Check source file
        assert_eq!(result[0].source_file, file_path.to_string_lossy().to_string());
        
        // Check that line numbers are set
        assert!(result[0].line_number.is_some());
        assert!(result[1].line_number.is_some());
        assert!(result[2].line_number.is_some());
    }

    #[test]
    fn test_process_files() {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let file1_path = temp_dir.path().join("file1.rst");
        let file2_path = temp_dir.path().join("file2.rst");
        
        // Create test RST files
        let rst_content1 = r#"
.. directive1::
   :option1: value1

   Content for directive1 in file1.
"#;
        
        let rst_content2 = r#"
.. directive2::
   :option2: value2

   Content for directive2 in file2.

.. directive1::
   :option3: value3

   Content for directive1 in file2.
"#;
        
        File::create(&file1_path).unwrap().write_all(rst_content1.as_bytes()).unwrap();
        File::create(&file2_path).unwrap().write_all(rst_content2.as_bytes()).unwrap();
        
        // Create processor to find directive1 and directive2
        let processor = Processor::new(vec!["directive1".to_string(), "directive2".to_string()]);
        let result = processor.process_files(vec![file1_path.clone(), file2_path.clone()]).unwrap();
        
        // Should find 3 directives in total
        assert_eq!(result.len(), 3);
        
        // Count directives by source file
        let file1_directives = result.iter().filter(|d| d.source_file == file1_path.to_string_lossy()).count();
        let file2_directives = result.iter().filter(|d| d.source_file == file2_path.to_string_lossy()).count();
        
        assert_eq!(file1_directives, 1);
        assert_eq!(file2_directives, 2);
        
        // Check that line numbers are set for all directives
        for directive in &result {
            assert!(directive.line_number.is_some());
        }
    }
}

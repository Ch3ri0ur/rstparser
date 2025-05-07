use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use serde::{Serialize, Deserialize};
use crate::parser::Directive;

/// A struct representing a directive with its source file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveWithSource {
    pub directive: Directive,
    pub source_file: String,
    pub line_number: Option<usize>, // Optional line number where the directive was found
}

/// A struct to handle aggregation of directives into JSON files
pub struct Aggregator {
    output_dir: PathBuf,
    group_by: GroupBy,
}

/// Enum to specify how directives should be grouped in output files
#[derive(Debug, Clone, Copy)]
pub enum GroupBy {
    /// Group by directive name (one JSON file per directive type)
    DirectiveName,
    /// Group all directives into a single JSON file
    All,
    /// Group by source file (one JSON file per source file)
    SourceFile,
}

impl Aggregator {
    /// Create a new Aggregator with the specified output directory
    pub fn new<P: AsRef<Path>>(output_dir: P, group_by: GroupBy) -> Self {
        Aggregator {
            output_dir: output_dir.as_ref().to_path_buf(),
            group_by,
        }
    }

    /// Aggregate directives and write them to JSON files
    pub fn aggregate_to_json(
        &self,
        directives: Vec<DirectiveWithSource>,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;
        
        let mut output_files = Vec::new();
        
        match self.group_by {
            GroupBy::DirectiveName => {
                // Group directives by name
                let mut grouped: HashMap<String, Vec<DirectiveWithSource>> = HashMap::new();
                
                for directive_with_source in directives {
                    let name = directive_with_source.directive.name.clone();
                    grouped.entry(name).or_insert_with(Vec::new).push(directive_with_source);
                }
                
                // Write each group to a separate file
                for (name, group) in grouped {
                    let file_path = self.output_dir.join(format!("{}.json", name));
                    fs::write(&file_path, serde_json::to_string_pretty(&group)?)?;
                    output_files.push(file_path);
                }
            },
            GroupBy::All => {
                // Write all directives to a single file
                let file_path = self.output_dir.join("all_directives.json");
                fs::write(&file_path, serde_json::to_string_pretty(&directives)?)?;
                output_files.push(file_path);
            },
            GroupBy::SourceFile => {
                // Group directives by source file
                let mut grouped: HashMap<String, Vec<DirectiveWithSource>> = HashMap::new();
                
                for directive_with_source in directives {
                    let source_file = directive_with_source.source_file.clone();
                    grouped.entry(source_file).or_insert_with(Vec::new).push(directive_with_source);
                }
                
                // Write each group to a separate file
                for (source_file, group) in grouped {
                    // Extract filename from path for the output file name
                    let file_name = Path::new(&source_file)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let file_path = self.output_dir.join(format!("{}.json", file_name));
                    fs::write(&file_path, serde_json::to_string_pretty(&group)?)?;
                    output_files.push(file_path);
                }
            },
        }
        
        Ok(output_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_aggregate_by_directive_name() {
        // Create a temporary directory for output
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path();
        
        // Create test directives
        let mut options1 = HashMap::new();
        options1.insert("option1".to_string(), "value1".to_string());
        
        let directive1 = Directive {
            name: "directive1".to_string(),
            arguments: "".to_string(),
            options: options1,
            content: "Content 1".to_string(),
        };
        
        let mut options2 = HashMap::new();
        options2.insert("option2".to_string(), "value2".to_string());
        
        let directive2 = Directive {
            name: "directive2".to_string(),
            arguments: "".to_string(),
            options: options2,
            content: "Content 2".to_string(),
        };
        
        let directive3 = Directive {
            name: "directive1".to_string(), // Same name as directive1
            arguments: "".to_string(),
            options: HashMap::new(),
            content: "Content 3".to_string(),
        };
        
        let directives_with_source = vec![
            DirectiveWithSource {
                directive: directive1,
                source_file: "file1.rst".to_string(),
                line_number: Some(10),
            },
            DirectiveWithSource {
                directive: directive2,
                source_file: "file2.rst".to_string(),
                line_number: Some(20),
            },
            DirectiveWithSource {
                directive: directive3,
                source_file: "file3.rst".to_string(),
                line_number: Some(30),
            },
        ];
        
        // Create aggregator and aggregate by directive name
        let aggregator = Aggregator::new(output_path, GroupBy::DirectiveName);
        let output_files = aggregator.aggregate_to_json(directives_with_source).unwrap();
        
        // Should create 2 files (one for each directive name)
        assert_eq!(output_files.len(), 2);
        
        // Check that the files exist
        let directive1_file = output_path.join("directive1.json");
        let directive2_file = output_path.join("directive2.json");
        
        assert!(directive1_file.exists());
        assert!(directive2_file.exists());
        
        // Read and parse the files to verify content
        let directive1_content: Vec<DirectiveWithSource> = 
            serde_json::from_str(&fs::read_to_string(directive1_file).unwrap()).unwrap();
        let directive2_content: Vec<DirectiveWithSource> = 
            serde_json::from_str(&fs::read_to_string(directive2_file).unwrap()).unwrap();
        
        // directive1.json should contain 2 directives
        assert_eq!(directive1_content.len(), 2);
        // directive2.json should contain 1 directive
        assert_eq!(directive2_content.len(), 1);
    }
    
    #[test]
    fn test_aggregate_all() {
        // Create a temporary directory for output
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path();
        
        // Create test directives
        let directive1 = Directive {
            name: "directive1".to_string(),
            arguments: "".to_string(),
            options: HashMap::new(),
            content: "Content 1".to_string(),
        };
        
        let directive2 = Directive {
            name: "directive2".to_string(),
            arguments: "".to_string(),
            options: HashMap::new(),
            content: "Content 2".to_string(),
        };
        
        let directives_with_source = vec![
            DirectiveWithSource {
                directive: directive1,
                source_file: "file1.rst".to_string(),
                line_number: Some(10),
            },
            DirectiveWithSource {
                directive: directive2,
                source_file: "file2.rst".to_string(),
                line_number: Some(20),
            },
        ];
        
        // Create aggregator and aggregate all directives
        let aggregator = Aggregator::new(output_path, GroupBy::All);
        let output_files = aggregator.aggregate_to_json(directives_with_source).unwrap();
        
        // Should create 1 file
        assert_eq!(output_files.len(), 1);
        
        // Check that the file exists
        let all_directives_file = output_path.join("all_directives.json");
        assert!(all_directives_file.exists());
        
        // Read and parse the file to verify content
        let content: Vec<DirectiveWithSource> = 
            serde_json::from_str(&fs::read_to_string(all_directives_file).unwrap()).unwrap();
        
        // Should contain 2 directives
        assert_eq!(content.len(), 2);
    }
    
    #[test]
    fn test_aggregate_by_source_file() {
        // Create a temporary directory for output
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path();
        
        // Create test directives
        let directive1 = Directive {
            name: "directive1".to_string(),
            arguments: "".to_string(),
            options: HashMap::new(),
            content: "Content 1".to_string(),
        };
        
        let directive2 = Directive {
            name: "directive2".to_string(),
            arguments: "".to_string(),
            options: HashMap::new(),
            content: "Content 2".to_string(),
        };
        
        let directive3 = Directive {
            name: "directive3".to_string(),
            arguments: "".to_string(),
            options: HashMap::new(),
            content: "Content 3".to_string(),
        };
        
        let directives_with_source = vec![
            DirectiveWithSource {
                directive: directive1,
                source_file: "file1.rst".to_string(),
                line_number: Some(10),
            },
            DirectiveWithSource {
                directive: directive2,
                source_file: "file1.rst".to_string(), // Same source file as directive1
                line_number: Some(20),
            },
            DirectiveWithSource {
                directive: directive3,
                source_file: "file2.rst".to_string(),
                line_number: Some(30),
            },
        ];
        
        // Create aggregator and aggregate by source file
        let aggregator = Aggregator::new(output_path, GroupBy::SourceFile);
        let output_files = aggregator.aggregate_to_json(directives_with_source).unwrap();
        
        // Should create 2 files (one for each source file)
        assert_eq!(output_files.len(), 2);
        
        // Check that the files exist
        let file1_output = output_path.join("file1.rst.json");
        let file2_output = output_path.join("file2.rst.json");
        
        assert!(file1_output.exists());
        assert!(file2_output.exists());
        
        // Read and parse the files to verify content
        let file1_content: Vec<DirectiveWithSource> = 
            serde_json::from_str(&fs::read_to_string(file1_output).unwrap()).unwrap();
        let file2_content: Vec<DirectiveWithSource> = 
            serde_json::from_str(&fs::read_to_string(file2_output).unwrap()).unwrap();
        
        // file1.rst.json should contain 2 directives
        assert_eq!(file1_content.len(), 2);
        // file2.rst.json should contain 1 directive
        assert_eq!(file2_content.len(), 1);
    }
}

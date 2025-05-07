use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::error::Error;
use std::ffi::OsStr;

/// A struct to configure file walking options
pub struct FileWalker {
    extensions: Vec<String>,
    max_depth: Option<usize>,
}

impl FileWalker {
    /// Create a new FileWalker with default settings
    pub fn new() -> Self {
        FileWalker {
            extensions: vec!["rst".to_string()], // Default to .rst files
            max_depth: None,                     // No depth limit by default
        }
    }

    /// Set the file extensions to filter by
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Set the maximum directory depth to traverse
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Walk the directory and find files with the specified extensions
    pub fn find_files<P: AsRef<Path>>(&self, root_dir: P) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut files = Vec::new();
        let mut walker = WalkDir::new(root_dir);
        
        // Apply max depth if specified
        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        for entry in walker.into_iter().filter_map(Result::ok) {
            let path = entry.path();
            
            // Skip directories
            if path.is_dir() {
                continue;
            }
            
            // Check if the file has one of the specified extensions
            if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                if self.extensions.iter().any(|e| e == ext) {
                    files.push(path.to_path_buf());
                }
            }
        }
        
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_find_rst_files() {
        // Create a temporary directory structure
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // Create some test files
        let file1_path = temp_path.join("file1.rst");
        let file2_path = temp_path.join("file2.txt");
        let file3_path = temp_path.join("subdir").join("file3.rst");
        
        // Create the subdirectory
        fs::create_dir(temp_path.join("subdir")).unwrap();
        
        // Create the files
        File::create(&file1_path).unwrap().write_all(b"test content").unwrap();
        File::create(&file2_path).unwrap().write_all(b"test content").unwrap();
        File::create(&file3_path).unwrap().write_all(b"test content").unwrap();
        
        // Test with default settings (only .rst files)
        let walker = FileWalker::new();
        let files = walker.find_files(temp_path).unwrap();
        
        // Should find 2 .rst files
        assert_eq!(files.len(), 2);
        assert!(files.contains(&file1_path));
        assert!(files.contains(&file3_path));
        assert!(!files.contains(&file2_path));
        
        // Test with custom extension
        let walker = FileWalker::new().with_extensions(vec!["txt".to_string()]);
        let files = walker.find_files(temp_path).unwrap();
        
        // Should find 1 .txt file
        assert_eq!(files.len(), 1);
        assert!(files.contains(&file2_path));
        
        // Test with max depth of 1 (no subdirectories)
        let walker = FileWalker::new().with_max_depth(1);
        let files = walker.find_files(temp_path).unwrap();
        
        // Should find only 1 .rst file in the root directory
        assert_eq!(files.len(), 1);
        assert!(files.contains(&file1_path));
        assert!(!files.contains(&file3_path));
    }
}

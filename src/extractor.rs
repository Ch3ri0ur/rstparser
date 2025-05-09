// Re-export RstExtractor from the library crate
pub use regex::Regex;
use std::path::Path;
use std::ffi::OsStr;

/// A struct to extract RST content from different file types
pub struct RstExtractor;

impl RstExtractor {
    /// Extract RST content from a file based on its extension
    pub fn extract_from_file<P: AsRef<Path>>(file_path: P, content: &str) -> String {
        let file_path = file_path.as_ref();
        
        match file_path.extension().and_then(OsStr::to_str) {
            Some("cpp") => Self::extract_from_cpp(content),
            Some("py") => Self::extract_from_python(content),
            _ => content.to_string(), // For .rst files, use the content as is
        }
    }

    /// Extract RST content from C++ files (between @rst and @endrst in /// comments)
    fn extract_from_cpp(content: &str) -> String {
        let mut result = String::new();
        
        // Match sequences of /// comments that contain @rst and @endrst markers
        // This pattern looks for:
        // 1. A line with /// that contains @rst
        // 2. Followed by any content until
        // 3. A line with /// that contains @endrst
        let re = Regex::new(r"(?s)///\s*@rst\b(.*?)///\s*@endrst\b").unwrap();
        
        for cap in re.captures_iter(content) {
            if let Some(rst_block) = cap.get(1) {
                // Process the captured RST content
                let processed_block = Self::process_cpp_rst_content(rst_block.as_str());
                
                // Add a separator if we already have content
                if !result.is_empty() {
                    result.push_str("\n\n");
                }
                
                result.push_str(&processed_block);
            }
        }
        
        result
    }

    /// Process RST content from C++ comments by removing /// prefixes and handling indentation
    fn process_cpp_rst_content(content: &str) -> String {
        // Remove /// prefixes and handle indentation
        let lines: Vec<&str> = content.lines().collect();
        
        // Find minimum indentation (ignoring empty lines)
        // We don't actually use this, but it's here for consistency with the Python version
        let _min_indent = lines.iter()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| {
                if let Some(_stripped) = line.trim_start().strip_prefix("///") {
                    Some(line.len() - line.trim_start().len())
                } else {
                    None
                }
            })
            .min()
            .unwrap_or(0);
        
        // Process each line
        let processed_lines: Vec<String> = lines.iter()
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    String::new()
                } else if let Some(stripped) = line.trim_start().strip_prefix("///") {
                    stripped.trim_start().to_string()
                } else {
                    trimmed.to_string()
                }
            })
            .collect();
        
        // Trim leading and trailing empty lines
        let mut result = processed_lines.join("\n");
        result = result.trim().to_string();
        
        result
    }

    /// Extract RST content from Python files (between @rst and @endrst in """ docstrings)
    fn extract_from_python(content: &str) -> String {
        let mut result = String::new();
        
        // Match any triple-quoted docstring that contains @rst and @endrst markers
        // This pattern looks for:
        // 1. Triple quotes (""")
        // 2. Any content until @rst
        // 3. Capture everything between @rst and @endrst
        // 4. Any content until the closing triple quotes
        let re = Regex::new(r#"(?s)""".*?@rst\b(.*?)@endrst\b.*?""""#).unwrap();
        
        for cap in re.captures_iter(content) {
            if let Some(rst_block) = cap.get(1) {
                // Process the captured RST content
                let processed_block = Self::process_python_rst_content(rst_block.as_str());
                
                // Add a separator if we already have content
                if !result.is_empty() {
                    result.push_str("\n\n");
                }
                
                result.push_str(&processed_block);
            }
        }
        
        result
    }

    /// Process RST content from Python docstrings by handling indentation
    fn process_python_rst_content(content: &str) -> String {
        // Handle indentation by finding the minimum indentation level
        let lines: Vec<&str> = content.lines().collect();
        
        // Find minimum indentation (ignoring empty lines)
        let min_indent = lines.iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);
        
        // Remove the minimum indentation from each line
        let processed_lines: Vec<String> = lines.iter()
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else if line.len() > min_indent {
                    line[min_indent..].to_string()
                } else {
                    line.trim_start().to_string()
                }
            })
            .collect();
        
        // Trim leading and trailing empty lines
        let mut result = processed_lines.join("\n");
        result = result.trim().to_string();
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_cpp() {
        let cpp_content = r#"
/// Some C++ code
/// 
/// @rst
/// This is RST content.
/// 
/// * Item 1
/// * Item 2
/// @endrst
///
/// More C++ code
"#;

        let expected = "This is RST content.\n\n* Item 1\n* Item 2";
        assert_eq!(RstExtractor::extract_from_cpp(cpp_content), expected);
    }

    #[test]
    fn test_extract_from_python() {
        let py_content = r#"
def some_function():
    """
    Some Python docstring
    
    @rst
    This is RST content.
    
    * Item 1
    * Item 2
    @endrst
    
    More docstring content
    """
    pass
"#;

        let expected = "This is RST content.\n\n* Item 1\n* Item 2";
        assert_eq!(RstExtractor::extract_from_python(py_content), expected);
    }

    #[test]
    fn test_multiple_rst_blocks_in_cpp() {
        let cpp_content = r#"
/// @rst
/// First RST block
/// @endrst
///
/// Some code
///
/// @rst
/// Second RST block
/// @endrst
"#;

        let expected = "First RST block\n\nSecond RST block";
        assert_eq!(RstExtractor::extract_from_cpp(cpp_content), expected);
    }

    #[test]
    fn test_multiple_rst_blocks_in_python() {
        let py_content = r#"
"""
@rst
First RST block
@endrst
"""

def some_function():
    """
    @rst
    Second RST block
    @endrst
    """
    pass
"#;

        let expected = "First RST block\n\nSecond RST block";
        assert_eq!(RstExtractor::extract_from_python(py_content), expected);
    }
}

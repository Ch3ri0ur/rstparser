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
    pub fn extract_from_cpp(content: &str) -> String {
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
        let raw_lines: Vec<&str> = content.lines().collect();

        if raw_lines.is_empty() {
            return String::new();
        }

        // Step 1 & 2: Strip C++ comment prefixes (e.g., "/// ", "///") and leading whitespace before them
        let stripped_lines: Vec<String> = raw_lines.iter()
            .map(|&line| {
                let mut current_line = line.trim_start(); // Strip whitespace before ///
                // Remove all leading slashes from the already trimmed line
                while current_line.starts_with('/') {
                    current_line = &current_line[1..];
                }
                // After removing slashes, any space immediately following "///" is preserved
                // as part of the RST content's own indentation.
                current_line.to_string()
            })
            .collect();

        // Step 3: Calculate common minimum indentation from the stripped lines
        let min_indent = stripped_lines.iter()
            .filter(|line_str| !line_str.trim().is_empty()) // Exclude lines that are now empty or only whitespace
            .map(|line_str| line_str.len() - line_str.trim_start().len()) // Indentation of the string after slash stripping
            .min()
            .unwrap_or(0);

        // Step 4: Remove common minimum indentation from stripped lines
        let processed_lines: Vec<String> = stripped_lines.iter()
            .map(|line_str| {
                if line_str.trim().is_empty() {
                    // For lines that became empty (or were whitespace-only) after initial stripping,
                    // ensure they are truly empty in the final output.
                    String::new()
                } else {
                    // Calculate current leading whitespace length for this line_str
                    let current_indent = line_str.len() - line_str.trim_start().len();
                    // Determine how many spaces to actually remove
                    let num_spaces_to_remove = std::cmp::min(current_indent, min_indent);
                    line_str[num_spaces_to_remove..].to_string()
                }
            })
            .collect();

        // Step 5 & 6: Join, trim, and return
        let mut result = processed_lines.join("\n");
        result = result.trim().to_string();
        
        result
    }

    /// Extract RST content from Python files (between @rst and @endrst in """ docstrings)
    pub fn extract_from_python(content: &str) -> String {
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
        // Step 1: Get raw lines from the content
        let raw_lines: Vec<&str> = content.lines().collect();
        
        if raw_lines.is_empty() {
            return String::new();
        }

        // Step 2: Calculate common minimum indentation from raw lines
        // (ignoring empty lines or lines with only whitespace)
        let min_indent = raw_lines.iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);
        
        // Step 3: Remove common minimum indentation from each line
        let processed_lines: Vec<String> = raw_lines.iter()
            .map(|line_str| { // line_str is &str
                if line_str.trim().is_empty() {
                    String::new() // Preserve intentionally empty lines as empty
                } else {
                    // Determine current indentation for this specific line
                    let current_indent = line_str.len() - line_str.trim_start().len();
                    // Calculate how many spaces to actually remove (cannot remove more than it has)
                    let num_spaces_to_remove = std::cmp::min(current_indent, min_indent);
                    line_str[num_spaces_to_remove..].to_string()
                }
            })
            .collect();
        
        // Step 4: Join processed lines, trim leading/trailing empty lines from the whole block, and return
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

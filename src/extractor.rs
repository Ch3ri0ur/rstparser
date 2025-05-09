use std::path::Path;
use std::ffi::OsStr;

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

        let expected = r#"This is RST content.

* Item 1
* Item 2"#;

        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_content),
            expected,
            "Regex C++ failed"
        )
    }

    #[test]
    fn test_extract_from_python() {
        let py_content = r#"
"""Some Python comment"""

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

        let expected = r"This is RST content.

* Item 1
* Item 2";

        assert_eq!(
            RstExtractor::extract_from_python(py_content),
            expected,
            "Regex Python failed"
        );
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

        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_content),
            expected,
            "Regex C++ multiple blocks failed"
        );
    }

    #[test]
    fn test_non_uniform_comments_in_cpp() {
        let cpp_content = r#"
// @rst
/// First RST block
// @endrst
///
/// Some code
///
 // @rst
/// Second RST block
/// @endrst
"#;

        let expected = "First RST block\n\nSecond RST block";

        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_content),
            expected,
            "Regex C++ multiple blocks failed"
        );
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
    @rst
    Third RST block
    @endrst
    """
    pass
"#;

        let expected = "First RST block\n\nSecond RST block\n\nThird RST block";

        assert_eq!(
            RstExtractor::extract_from_python(py_content),
            expected,
            "Regex Python multiple blocks failed"
        );
    }

    #[test]
    fn test_extract_from_cpp_indentation() {
        let cpp_content_standard = r#"
/// Some C++ code
/// 
/// @rst
/// This is RST content.
/// 
///  * Item 1
///  * Item 2
/// @endrst
///
/// More C++ code
/// @rst
/// Second block
///  Indented.
/// @endrst
"#;
        let expected_standard =
            "This is RST content.\n\n * Item 1\n * Item 2\n\nSecond block\n Indented.";
        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_content_standard),
            expected_standard,
            "Regex C++ standard case failed"
        );
    }

    #[test]
    fn test_extract_from_cpp_single_line() {
        let cpp_single_line_rst = r#"/// @rst Message @endrst"#;
        let expected_single_line = "Message";

        // This test is not necessariy. It is an optional edge case which can also be ignored.
        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_single_line_rst),
            expected_single_line,
            "Manual C++ single line failed"
        );
    }

    #[test]
    fn test_extract_from_cpp_variants() {
        let cpp_content_mixed_indent = r#"
        ///    @rst
        ///      Line 1
        ///    Line 2
        /// @endrst
        "#;
        let expected_mixed_indent = "  Line 1\nLine 2";
        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_content_mixed_indent),
            expected_mixed_indent,
            "Regex C++ mixed indent failed"
        );
    }

    #[test]
    fn test_extract_from_python_variants() {
        let py_content_standard = r#"
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
        let expected_standard = "This is RST content.\n\n* Item 1\n* Item 2";
        assert_eq!(
            RstExtractor::extract_from_python(py_content_standard),
            expected_standard,
            "Regex Python standard case failed"
        );
    }

    #[test]
    fn test_extract_from_python_multiple_one_docstring() {
        let py_content_multiple_in_one_docstring = r#"
def third_function():
    """
    @rst
    Block one.
    @endrst
    Some text between.
    @rst
    Block two.
    @endrst
    """
    pass
"#;
        let expected_multiple_in_one = "Block one.\n\nBlock two.";


        assert_eq!(
            RstExtractor::extract_from_python(py_content_multiple_in_one_docstring),
            expected_multiple_in_one,
            "Regex Python multiple in one docstring failed"
        );
    }

    #[test]
    fn test_extract_from_python_indentation() {
        let py_content_multiple_docstrings = r#"
"""
@rst
First RST block
@endrst
"""

def some_function_multiple():
    """
    @rst
    Second RST block
      indented stuff
    non indented
    @endrst
    """
    pass
"#;
        let expected_multiple_docstrings =
            "First RST block\n\nSecond RST block\n  indented stuff\nnon indented";
        assert_eq!(
            RstExtractor::extract_from_python(py_content_multiple_docstrings),
            expected_multiple_docstrings,
            "Regex Python multiple docstrings failed"
        );
    }

    #[test]
    fn test_extract_from_python_marker_directly() {
        let py_rst_ends_docstring = r#"
"""@rst
Content that ends with docstring
@endrst""" 
"#; 
        let expected_ends_docstring = "Content that ends with docstring";
        assert_eq!(
            RstExtractor::extract_from_python(py_rst_ends_docstring),
            expected_ends_docstring,
            "Regex Python RST ends docstring failed"
        );
    }

    #[test]
    fn test_extract_from_python_no_rst() {
        let py_no_rst_in_docstring = r#"
"""
This is a docstring.
No RST here.
"""
"#;
        let expected_no_rst = "";
        assert_eq!(
            RstExtractor::extract_from_python(py_no_rst_in_docstring),
            expected_no_rst,
            "Regex Python no RST failed"
        );
    }

    #[test]
    fn test_extract_from_python_with_empty_lines() {
        let py_rst_with_empty_lines_inside = r#"
"""
@rst
Line 1

Line 3
@endrst

stuff

@rst
        Line 1

    Line 3
@endrst
"""
"#;
        let expected_empty_lines_inside = "Line 1\n\nLine 3\n\n    Line 1\n\nLine 3";
        assert_eq!(
            RstExtractor::extract_from_python(py_rst_with_empty_lines_inside),
            expected_empty_lines_inside,
            "Regex Python empty lines inside failed"
        );
    }

    #[test]
    fn test_cpp_empty_and_no_rst() {
        // Renamed
        let expected = "";
        assert_eq!(
            RstExtractor::extract_from_cpp(""),
            expected,
            "Regex C++ empty string failed"
        );

        assert_eq!(
            RstExtractor::extract_from_cpp("/// no rst here"),
            expected,
            "Regex C++ no rst failed"
        );

        // For unterminated, all methods should ideally return empty or handle gracefully.
        // It would good to throw a warning. in this case.
        assert_eq!(
            RstExtractor::extract_from_cpp("/// @rst unterminated"),
            expected,
            "Regex C++ unterminated failed"
        );
    }

    #[test]
    fn test_python_empty_and_no_rst() {
        // Renamed
        let expected = "";
        assert_eq!(
            RstExtractor::extract_from_python(""),
            expected,
            "Regex Python empty string failed"
        );

        assert_eq!(
            RstExtractor::extract_from_python("\"\"\"no rst here\"\"\""),
            expected,
            "Regex Python no rst failed"
        );

        // For unterminated, all methods should return empty.
        // method prints a warning.
        assert_eq!(
            RstExtractor::extract_from_python("\"\"\"@rst unterminated\"\"\""),
            expected,
            "Regex Python unterminated failed"
        );
    }
}

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


    pub fn extract_from_python(content: &str) -> String {
        let mut extracted_blocks = Vec::new();
        let mut current_pos = 0;

        // We need to find docstrings
        // While in a docstring we need to find @rst
        // while in a block we need to find @endrst
        // once a block is found we need to uniformly dedent the lines (ignoring empty lines)
        // repeat

    
        extracted_blocks.join("\n\n")
    }

    pub fn extract_from_cpp(content: &str) -> String {
        let mut extracted_blocks = Vec::new();
        let mut current_pos = 0;

        // We need to find commented lines
        // If we find uncommented code we reset
        // Inside a commented block we need to find @rst
        // while in a block we need to find @endrst
        // once a block is found we need to 
        // - remove all leading whitespaces
        // - remove all leading '/' 
        // - uniformly dedent the lines (ignoring empty lines)
        // repeat

        


        extracted_blocks.join("\n\n")
    }
}

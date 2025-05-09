use std::path::Path;
use std::ffi::OsStr;

// Helper function to uniformly dedent lines
fn dedent_lines(lines: Vec<String>) -> String {
    if lines.is_empty() {
        return String::new();
    }

    let mut min_indent = usize::MAX;
    for line in &lines {
        if line.trim().is_empty() {
            continue; // Skip empty lines for indent calculation
        }
        let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
        if leading_spaces < min_indent {
            min_indent = leading_spaces;
        }
    }

    if min_indent == usize::MAX { // All lines were empty or whitespace
        return lines.join("\n"); // Should be an empty string if lines is empty, or lines joined by \n
    }
    
    let mut processed_lines = Vec::new();
    for line in lines { // consume lines
        if line.trim().is_empty() {
            processed_lines.push(String::new()); // Preserve empty lines as empty strings
        } else if line.len() >= min_indent {
            processed_lines.push(line[min_indent..].to_string());
        } else {
            processed_lines.push(line); // Should not happen
        }
    }
    
    // Smart join:
    if processed_lines.is_empty() {
        return String::new();
    }
    // Remove empty lines from the beginning and end of the result
    while processed_lines.first().map_or(false, |line| line.trim().is_empty()) {
        processed_lines.remove(0);
    }
    while processed_lines.last().map_or(false, |line| line.trim().is_empty()) {
        processed_lines.pop();
    }
    

    let mut result = String::new();
    for (i, line) in processed_lines.iter().enumerate() {
        
        result.push_str(line);
        if i < processed_lines.len() - 1 {
            result.push('\n');
        }
    }
    result
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

        let expected = r#"This is RST content.

* Item 1
* Item 2"#;

        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_content),
            expected,
            "C++ basic extraction failed"
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
            "Python basic extraction failed"
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
            "C++ multiple blocks failed"
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
            "C++ non-uniform comments failed"
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
            "Python multiple blocks failed"
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
            "C++ indentation failed"
        );
    }

    #[test]
    fn test_extract_from_cpp_single_line() {
        let cpp_single_line_rst = r#"/// @rst Message @endrst"#;
        let expected_single_line = "Message";

        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_single_line_rst),
            expected_single_line,
            "C++ single line failed"
        );
    }
    
    #[test]
    fn test_extract_from_cpp_single_line_with_leading_comment_space() {
        let cpp_single_line_rst = r#"/// @rst Message @endrst"#;
        let expected_single_line = "Message";
        assert_eq!(
            RstExtractor::extract_from_cpp(cpp_single_line_rst),
            expected_single_line,
            "C++ single line with leading comment space failed"
        );

        let cpp_single_line_rst_no_space = r#"//@rst Message @endrst"#;
         assert_eq!(
            RstExtractor::extract_from_cpp(cpp_single_line_rst_no_space),
            expected_single_line,
            "C++ single line no leading comment space failed"
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
            "C++ mixed indent failed"
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
            "Python standard case failed"
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
            "Python multiple in one docstring failed"
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
            "Python multiple docstrings failed"
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
            "Python RST ends docstring failed"
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
            "Python no RST failed"
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
        // let expected_empty_lines_inside = "Line 1\n\nLine 3\n\n        Line 1\n\n    Line 3"; // Note: dedent logic will adjust the second block
        let dedented_expected_empty_lines_inside = "Line 1\n\nLine 3\n\n    Line 1\n\nLine 3";
        assert_eq!(
            RstExtractor::extract_from_python(py_rst_with_empty_lines_inside),
            dedented_expected_empty_lines_inside,
            "Python empty lines inside failed"
        );
    }

    #[test]
    fn test_cpp_empty_and_no_rst() {
        let expected = "";
        assert_eq!(
            RstExtractor::extract_from_cpp(""),
            expected,
            "C++ empty string failed"
        );

        assert_eq!(
            RstExtractor::extract_from_cpp("/// no rst here"),
            expected,
            "C++ no rst failed"
        );

        assert_eq!(
            RstExtractor::extract_from_cpp("/// @rst unterminated"),
            expected,
            "C++ unterminated failed"
        );
    }

    #[test]
    fn test_python_empty_and_no_rst() {
        let expected = "";
        assert_eq!(
            RstExtractor::extract_from_python(""),
            expected,
            "Python empty string failed"
        );

        assert_eq!(
            RstExtractor::extract_from_python("\"\"\"no rst here\"\"\""),
            expected,
            "Python no rst failed"
        );

        assert_eq!(
            RstExtractor::extract_from_python("\"\"\"@rst unterminated\"\"\""),
            expected,
            "Python unterminated failed"
        );
    }

    #[test]
    fn test_python_rst_at_start_and_end_of_docstring() {
        let content = r#"
"""@rst
Block one
@endrst"""
"#;
        let expected = "Block one";
        assert_eq!(RstExtractor::extract_from_python(content), expected, "Python RST at start/end of docstring");
    }

    #[test]
    fn test_python_rst_with_optional_newlines_which_should_be_removed() {
        let content = r#"
"""
@rst

Block one with newlines

@endrst
"""
"#;
        let expected = "Block one with newlines";
         assert_eq!(RstExtractor::extract_from_python(content), expected, "Python RST with optional newlines");
    }
}

pub struct RstExtractor;

impl RstExtractor {
    /// Extract RST content from a file based on its extension
    pub fn extract_from_file<P: AsRef<Path>>(file_path: P, content: &str) -> String {
        let file_path = file_path.as_ref();
        
        match file_path.extension().and_then(OsStr::to_str) {
            Some("cpp") | Some("h") | Some("hpp") | Some("cxx") | Some("hxx") | Some("cc") | Some("hh") => Self::extract_from_cpp(content),
            Some("py") => Self::extract_from_python(content),
            Some("rst") => content.to_string(), // For .rst files, use the content as is
            _ => {
                // eprint!("Unsupported file type for RST extraction: {:?}", file_path.extension());
                String::new() // Or return content.to_string() if unknown types should pass through
            }
        }
    }

    pub fn extract_from_python(content: &str) -> String {
        let mut extracted_blocks = Vec::new();
        let mut search_offset = 0;

        const TRIPLE_DOUBLE_QUOTE: &str = "\"\"\"";
        const TRIPLE_SINGLE_QUOTE: &str = "'''";
        const RST_START_MARKER: &str = "@rst";
        const RST_END_MARKER: &str = "@endrst";

        while search_offset < content.len() {
            let q1_start = content[search_offset..].find(TRIPLE_DOUBLE_QUOTE);
            let q3_start = content[search_offset..].find(TRIPLE_SINGLE_QUOTE);

            let (doc_start_marker, doc_start_rel) = match (q1_start, q3_start) {
                (Some(s1), Some(s3)) => {
                    if s1 <= s3 { (TRIPLE_DOUBLE_QUOTE, s1) } else { (TRIPLE_SINGLE_QUOTE, s3) }
                }
                (Some(s1), None) => (TRIPLE_DOUBLE_QUOTE, s1),
                (None, Some(s3)) => (TRIPLE_SINGLE_QUOTE, s3),
                (None, None) => break, // No more docstrings
            };
            
            let doc_start_abs = search_offset + doc_start_rel;
            let doc_content_start_abs = doc_start_abs + doc_start_marker.len();

            if let Some(doc_end_rel) = content[doc_content_start_abs..].find(doc_start_marker) {
                let doc_end_abs = doc_content_start_abs + doc_end_rel;
                let doc_content = &content[doc_content_start_abs..doc_end_abs];
                search_offset = doc_end_abs + doc_start_marker.len();

                let mut rst_search_offset_in_doc = 0;
                while rst_search_offset_in_doc < doc_content.len() {
                    if let Some(rst_start_rel) = doc_content[rst_search_offset_in_doc..].find(RST_START_MARKER) {
                        let rst_content_actual_start = rst_search_offset_in_doc + rst_start_rel + RST_START_MARKER.len();
                        if let Some(rst_end_rel) = doc_content[rst_content_actual_start..].find(RST_END_MARKER) {
                            let rst_content_actual_end = rst_content_actual_start + rst_end_rel;
                            let block_content_raw = &doc_content[rst_content_actual_start..rst_content_actual_end];
                            
                            let mut processed_block_str = block_content_raw;

                            // Check for trailing newline (and potential following spaces on that line)
                            // This needs to be done *after* leading newline is stripped if both are present.
                            if processed_block_str.ends_with('\n') {
                                processed_block_str = &processed_block_str[..processed_block_str.len() -1];
                                if processed_block_str.ends_with('\r') { // Handle \r\n specifically
                                    processed_block_str = &processed_block_str[..processed_block_str.len() -1];
                                }
                            } else if processed_block_str.ends_with("\r\n") {
                                 processed_block_str = &processed_block_str[..processed_block_str.len() -2];
                            }
                            
                            // After stripping optional newlines, if processed_block_str is empty,
                            // it means the original block was like "@rst\n@endrst" or "@rst @endrst" or "@rst@endrst"
                            if processed_block_str.is_empty() {
                                // If original block_content_raw was just newlines, it should be a block with one empty line.
                                // If block_content_raw was empty or just whitespace, it's an empty block.
                                if block_content_raw.trim().is_empty() && !block_content_raw.is_empty() { // e.g. @rst \n @endrst
                                    extracted_blocks.push(dedent_lines(vec![String::new()]));
                                } else { // e.g. @rst@endrst or @rst   @endrst
                                    extracted_blocks.push(String::new());
                                }
                            } else {
                                let lines_vec: Vec<String> = processed_block_str.lines().map(String::from).collect();
                                extracted_blocks.push(dedent_lines(lines_vec));
                            }
                            rst_search_offset_in_doc = rst_content_actual_end + RST_END_MARKER.len();
                        } else {
                            eprintln!("Warning: Unterminated RST block in Python docstring (missing @endrst).");
                            break; // Missing @endrst in this doc_content
                        }
                    } else {
                        break; // No more @rst in this doc_content
                    }
                }
            } else {
                eprintln!("Warning: Unterminated Python docstring.");
                break; // Unterminated docstring
            }
        }
        extracted_blocks.join("\n\n")
    }

    pub fn extract_from_cpp(content: &str) -> String {
        let mut extracted_blocks = Vec::new();
        let mut current_block_lines: Vec<String> = Vec::new();
        let mut in_rst_block = false;

        const RST_START_MARKER: &str = "@rst";
        const RST_END_MARKER: &str = "@endrst";

        for line in content.lines() {
            let trimmed_line = line.trim_start();
            let mut comment_content: Option<String> = None;

            if trimmed_line.starts_with("/// ") {
                comment_content = Some(trimmed_line["/// ".len()..].to_string());
            } else if trimmed_line.starts_with("///") { // No space after marker
                comment_content = Some(trimmed_line["///".len()..].to_string());
            } else if trimmed_line.starts_with("// ") {
                comment_content = Some(trimmed_line["// ".len()..].to_string());
            } else if trimmed_line.starts_with("//") { // No space after marker
                comment_content = Some(trimmed_line["//".len()..].to_string());
            }

            if in_rst_block {
                if let Some(text_in_comment) = comment_content.take() { // text_in_comment is the String from the comment line
                    // Check if this line terminates the RST block
                    if let Some(end_marker_pos) = text_in_comment.find(RST_END_MARKER) {
                        // This line contains @endrst.
                        let content_before_end_marker = text_in_comment[..end_marker_pos].trim_end();
                        if !content_before_end_marker.is_empty() {
                            current_block_lines.push(content_before_end_marker.to_string());
                        }

                        // Finalize current block
                        if !current_block_lines.is_empty() {
                            extracted_blocks.push(dedent_lines(current_block_lines.drain(..).collect::<Vec<String>>()));
                        }
                        in_rst_block = false;
                    } else {
                        // Line is a comment and part of the RST block content
                        current_block_lines.push(text_in_comment);
                    }
                } else {
                    // Non-comment line or empty line breaks the RST block
                    if line.trim().is_empty() && !current_block_lines.is_empty() {
                         // Preserve empty lines within a block if they are truly empty
                        current_block_lines.push(String::new());
                    } else if !line.trim().is_empty() {
                        eprintln!("Warning: Unterminated RST block in C++ content, broken by non-comment line: '{}'", line);
                        current_block_lines.clear();
                        in_rst_block = false;
                    } else if line.trim().is_empty() && current_block_lines.is_empty() && in_rst_block {
                        // If we are in a block, and it's an empty line, and we have no content yet,
                        // this could be the optional newline after @rst. Add it.
                        current_block_lines.push(String::new());
                    }
                }
            } else {
                if let Some(text_after_comment_marker) = comment_content.take() {
                    let potential_rst_line_content = text_after_comment_marker.trim_start(); // Trim spaces like "   @rst"
                    if potential_rst_line_content.starts_with(RST_START_MARKER) {
                        in_rst_block = true;
                        
                        let mut content_on_rst_line = potential_rst_line_content[RST_START_MARKER.len()..].to_string();
                        if content_on_rst_line.starts_with(' ') {
                            content_on_rst_line = content_on_rst_line[1..].to_string();
                        }
                        
                        // Check for @endrst on the same line
                        if let Some(end_marker_pos) = content_on_rst_line.find(RST_END_MARKER) {
                            let single_line_rst = content_on_rst_line[..end_marker_pos].trim_end_matches(' ').to_string();
                            if !single_line_rst.is_empty() {
                                extracted_blocks.push(single_line_rst);
                            } else if content_on_rst_line[..end_marker_pos].is_empty() && end_marker_pos == 0 {
                                extracted_blocks.push(String::new()); 
                            }
                            in_rst_block = false; 
                        } else {
                            // Content on the @rst line, after @rst and optional space
                            if !content_on_rst_line.is_empty() {
                                current_block_lines.push(content_on_rst_line);
                            }
                        }
                    }
                }
            }
        }

        if in_rst_block {
            eprintln!("Warning: Unterminated RST block at end of C++ content.");
            // current_block_lines.clear(); // As per test expectations for unterminated blocks
        }
        extracted_blocks.join("\n\n")
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub name: String,
    pub arguments: String,
    pub options: HashMap<String, String>,
    pub content: String,
}

/// Parses the body of a directive, given the text slice that starts immediately *after*
/// the ".. directive_name::" marker.
///
/// # Arguments
/// * `text_after_marker` - The text slice beginning with the directive's arguments (if any)
///                         on the first line, followed by options and content.
/// * `directive_name` - The name of the directive being parsed.
fn parse_directive_body(
    text_after_marker: &str,
    directive_name: String,
) -> Directive {
    let mut options = HashMap::new();
    let mut content_lines = Vec::new();
    let mut in_options = true;

    // Extract arguments - everything from the start of text_after_marker to the end of its first line
    let first_line_end = text_after_marker
        .find('\n')
        .map_or(text_after_marker.len(), |pos| pos);
    let arguments = text_after_marker[..first_line_end].trim().to_string();

    let mut block_indentation: Option<usize> = None;

    // Determine block_indentation from the first non-empty line after the argument line.
    let mut temp_lines_iter = text_after_marker.lines().skip(1).peekable(); // Skip argument line
    while let Some(line_str) = temp_lines_iter.next() {
        let trimmed_line_for_indent_check = line_str.trim_start();
        if !trimmed_line_for_indent_check.is_empty() {
            block_indentation = Some(line_str.len() - trimmed_line_for_indent_check.len());
            break;
        }
    }

    let mut lines_iter = text_after_marker.lines().skip(1).peekable(); // Skip argument line

    while let Some(line_str) = lines_iter.next() {
        let original_line_for_content = line_str.to_string();
        let current_indentation = line_str.len() - line_str.trim_start().len();
        let trimmed_line = line_str.trim();

        if in_options {
            if trimmed_line.starts_with(':') {
                let option_line_indentation = current_indentation;
                let mut parts_iter = trimmed_line[1..].splitn(2, ':');
                if let (Some(key_str), Some(value_str)) = (parts_iter.next(), parts_iter.next()) {
                    let key = key_str.trim().to_string();
                    let mut value_parts = vec![value_str.trim_start().to_string()];

                    loop {
                        match lines_iter.peek() {
                            Some(next_line_peek_str) => {
                                let next_line_original = *next_line_peek_str;
                                let next_line_indent = next_line_original.len()
                                    - next_line_original.trim_start().len();
                                let next_trimmed_line = next_line_original.trim();

                                // If the next line looks like a new option, stop collecting for current option's value
                                if next_trimmed_line.starts_with(':') && next_trimmed_line[1..].contains(':') {
                                    // Check if it's indented enough to be part of *this* directive's options,
                                    // or if it's less indented (could be a new directive or unrelated text)
                                    // For now, any new valid option format line terminates current option value.
                                    break;
                                }

                                if !next_trimmed_line.is_empty()
                                    && next_line_indent > option_line_indentation
                                {
                                    value_parts.push(next_trimmed_line.to_string());
                                    lines_iter.next(); 
                                } else {
                                    break; 
                                }
                            }
                            None => break,
                        }
                    }
                    let final_value = if value_parts.len() > 1 && value_parts[0].is_empty() {
                        value_parts[1..].join("\n")
                    } else {
                        value_parts.join("\n")
                    };
                    options.insert(key, final_value);
                    continue;
                } else {
                    in_options = false;
                }
            } else {
                in_options = false;
                if trimmed_line.is_empty() {
                    continue; 
                }
            }
        }

        if trimmed_line.starts_with(".. ") && trimmed_line.contains("::") {
            break;
        }

        let part_of_content_block = block_indentation.map_or(
            true, 
            |indent| current_indentation >= indent || trimmed_line.is_empty(),
        );

        if part_of_content_block {
            content_lines.push(original_line_for_content);
        } else if !trimmed_line.is_empty() {
            break;
        }
    }

    let mut min_indent: Option<usize> = None;
    for line in &content_lines {
        if !line.trim().is_empty() {
            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();
            min_indent = match min_indent {
                Some(indent) => Some(std::cmp::min(indent, current_indent)),
                None => Some(current_indent),
            };
        }
    }

    let mut processed_content_lines: Vec<String> = content_lines
        .into_iter()
        .map(|line| {
            if line.trim().is_empty() {
                "".to_string()
            } else {
                match min_indent {
                    Some(indent) => line.chars().skip(indent).collect::<String>(),
                    None => line,
                }
            }
        })
        .collect();

    while processed_content_lines
        .last()
        .map_or(false, |l| l.trim().is_empty())
    {
        processed_content_lines.pop();
    }

    Directive {
        name: directive_name,
        arguments,
        options,
        content: processed_content_lines.join("\n"),
    }
}

// Helper function to check for valid directive name characters.
// Directive names cannot contain spaces themselves.
// Standard RST allows alphanumeric, hyphen, underscore, period.
fn is_valid_directive_char_for_name(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
    // No space allowed here based on user feedback for strict RST.
}

/// Parse a reStructuredText string and find all occurrences of any directive in the provided list.
/// Performs a single pass over the text for efficiency.
/// Returns a vector of all found directives with their line numbers, in the order they appear.
pub fn parse_rst_multiple(text: &str, target_directives: &[&str]) -> Vec<(Directive, usize)> {
    let mut found_directives_with_pos = Vec::new();
    let mut current_pos = 0;

    while current_pos < text.len() {
        // Find the next potential directive start ".. " (must have a space)
        if let Some(dots_space_offset) = text[current_pos..].find(".. ") {
            let absolute_dots_space_start = current_pos + dots_space_offset;
            let potential_directive_line_start = absolute_dots_space_start;
            let name_search_start_abs = absolute_dots_space_start + 3; // Name starts after ".. "

            // Minimum length for a directive: ".. a::" (6 chars)
            if name_search_start_abs >= text.len() || absolute_dots_space_start + 6 > text.len() {
                break;
            }

            // Determine the end of the current line for searching "::"
            let end_of_line_offset_from_name_start = text[name_search_start_abs..]
                .find('\n')
                .map_or(text.len() - name_search_start_abs, |pos| pos);

            let line_search_slice = &text[name_search_start_abs..name_search_start_abs + end_of_line_offset_from_name_start];

            if let Some(colon_colon_offset_in_slice) = line_search_slice.find("::") {
                let absolute_colon_colon_start = name_search_start_abs + colon_colon_offset_in_slice;
                let directive_name_candidate_str = &text[name_search_start_abs..absolute_colon_colon_start];
                let trimmed_name = directive_name_candidate_str.trim(); // Trim spaces around the name

                // Validate directive name characters (no spaces within the name itself)
                let is_name_structurally_valid = !trimmed_name.is_empty() &&
                    !trimmed_name.contains(' ') && // Ensure no internal spaces in the name
                    trimmed_name.chars().all(is_valid_directive_char_for_name);

                if is_name_structurally_valid && target_directives.contains(&trimmed_name) {
                    let line_number = text[..potential_directive_line_start].matches('\n').count() + 1;
                    let directive_body_start_index = absolute_colon_colon_start + 2; // After "::"

                    if directive_body_start_index <= text.len() {
                        let directive = parse_directive_body(
                            &text[directive_body_start_index..],
                            trimmed_name.to_string(),
                        );
                        found_directives_with_pos.push((potential_directive_line_start, directive, line_number));
                    }
                    current_pos = directive_body_start_index;
                } else {
                    // Invalid name, not a target, or malformed, but "::" was found after ".. ".
                    // Advance past this "::" to avoid reprocessing.
                    current_pos = absolute_colon_colon_start + 2;
                }
            } else {
                // Found ".. " but no "::" on the same line after the name part.
                // Advance past the ".. " to continue searching.
                current_pos = name_search_start_abs; // which is absolute_dots_space_start + 3
            }
        } else {
            // No more ".. " found
            break;
        }
    }

    found_directives_with_pos
        .into_iter()
        .map(|(_, directive, line_number)| (directive, line_number))
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Helper to create a HashMap for options easily in tests
    fn opts(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
    
    // Test assertion helper for a single expected directive
    fn assert_single_directive_eq_props(
        results: &Vec<(Directive, usize)>,
        expected_name: &str,
        expected_arguments: &str,
        expected_options: &HashMap<String, String>,
        expected_content: &str,
        expected_line: Option<usize>,
    ) {
        assert_eq!(results.len(), 1, "Expected 1 directive, found {}", results.len());
        let (directive, line_number) = &results[0];
        assert_eq!(directive.name, expected_name.to_string(), "Name mismatch");
        assert_eq!(directive.arguments, expected_arguments.to_string(), "Argument mismatch");
        assert_eq!(&directive.options, expected_options, "Options mismatch");
        assert_eq!(directive.content, expected_content.to_string(), "Content mismatch");
        if let Some(line) = expected_line {
            assert_eq!(*line_number, line, "Line number mismatch");
        }
    }

    // Test assertion helper for expecting no directives
    fn assert_no_directives_found(results: &Vec<(Directive, usize)>, directive_name_searched: &str) {
        assert!(results.is_empty(), "Expected no directives for '{}', found {} ({:?})", directive_name_searched, results.len(), results);
    }

    #[test]
    fn test_basic_directive() {
        let rst = r#"
.. mydirective::
   :option1: value1
   :option2: value2

   This is content.
"#;
        let expected_options = opts(&[("option1", "value1"), ("option2", "value2")]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "This is content.",
            Some(2),
        );
    }

    #[test]
    fn test_directive_no_options() {
        let rst = r#"
.. mydirective::

   This is content without options.
"#;
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &HashMap::new(),
            "This is content without options.",
            Some(2),
        );
    }
    
    #[test]
    fn test_directive_no_content() {
        let rst = r#"
.. mydirective::
   :option1: value1
"#;
        let expected_options = opts(&[("option1", "value1")]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "",
            Some(2),
        );
    }

    #[test]
    fn test_directive_no_options_no_content_trailing_newline() {
        let rst = ".. mydirective::\n";
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &HashMap::new(),
            "",
            Some(1),
        );
    }

    #[test]
    fn test_directive_no_options_no_content_no_trailing_newline() {
        let rst = ".. mydirective::";
        let results = parse_rst_multiple(rst, &["mydirective"]);
         assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &HashMap::new(),
            "",
            Some(1),
        );
    }

    #[test]
    fn test_directive_with_hyphens_underscores() {
        let rst = r#"
.. my-directive_name::
   :option-key_1: value_with-hyphen

   Content here.
"#;
        let expected_options = opts(&[("option-key_1", "value_with-hyphen")]);
        let results = parse_rst_multiple(rst, &["my-directive_name"]);
        assert_single_directive_eq_props(
            &results,
            "my-directive_name",
            "",
            &expected_options,
            "Content here.",
            Some(2),
        );
    }

    #[test]
    fn test_multiple_directives_with_parse_rst_multiple() { // Corrected test name
        let rst = r#"
.. first_directive::
   :key1: val1

   Content for first.

.. second_directive::
   :key2: val2

   Content for second.
"#;
        let results_first = parse_rst_multiple(rst, &["first_directive"]);
        let expected_options1 = opts(&[("key1", "val1")]);
        assert_single_directive_eq_props(
            &results_first,
            "first_directive",
            "",
            &expected_options1,
            "Content for first.",
            Some(2),
        );

        let results_second = parse_rst_multiple(rst, &["second_directive"]);
        let expected_options2 = opts(&[("key2", "val2")]);
         assert_single_directive_eq_props(
            &results_second,
            "second_directive",
            "",
            &expected_options2,
            "Content for second.",
            Some(7), 
        );
    }

    #[test]
    fn test_directive_not_found() {
        let rst = r#"
.. existing_directive::
   :k: v

   Some text.
"#;
        let results = parse_rst_multiple(rst, &["nonexistent_directive"]);
        assert_no_directives_found(&results, "nonexistent_directive");
    }

    #[test]
    fn test_empty_input_string() {
        let results = parse_rst_multiple("", &["anydirective"]);
        assert_no_directives_found(&results, "anydirective");
    }

    #[test]
    fn test_content_starts_immediately_after_directive_line() {
        let rst = r#"
.. mydirective::
   Immediately starting content.
   More content.
"#;
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &HashMap::new(),
            "Immediately starting content.\nMore content.",
            Some(2),
        );
    }

    #[test]
    fn test_content_starts_immediately_after_options_no_blank_line() {
        let rst = r#"
.. mydirective::
   :option1: value1
   This is content starting right after an option line.
   Another line of content.
"#;
        let expected_options = opts(&[("option1", "value1")]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "This is content starting right after an option line.\nAnother line of content.",
            Some(2),
        );
    }
    
    #[test]
    fn test_malformed_option_missing_second_colon() {
        let rst = r#"
.. mydirective::
   :option1 value1
   :option2: value2

   Content
"#;
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &HashMap::new(), 
            ":option1 value1\n:option2: value2\n\nContent", // Content parsing is greedy
            Some(2),
        );
    }

    #[test]
    fn test_malformed_option_empty_value_after_colon() {
        let rst = r#"
.. mydirective::
   :option1:
   :option2: value2

   Content
"#;
        let expected_options = opts(&[("option1", ""), ("option2", "value2")]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "Content",
            Some(2),
        );
    }

    #[test]
    fn test_directive_at_end_of_file_with_content() {
        let rst = ".. mydirective::\n\n   Final content.";
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &HashMap::new(),
            "Final content.",
            Some(1),
        );
    }

    #[test]
    fn test_directive_at_end_of_file_with_options() {
        let rst = ".. mydirective::\n   :key: val";
        let expected_options = opts(&[("key", "val")]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "",
            Some(1),
        );
    }
    
    #[test]
    fn test_options_with_various_spacing() {
        let rst = r#"
.. mydirective::
   :key1:value1
   :key2 : value2
   : key3 :value3
   :  key4  :  value4  

   Content
"#;
        let expected_options = opts(&[
            ("key1", "value1"), 
            ("key2", "value2"), 
            ("key3", "value3"), 
            ("key4", "value4")
        ]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "Content",
            Some(2),
        );
    }

    #[test]
    fn test_directive_followed_immediately_by_another() {
        let rst = r#"
.. first::
   :opt1: val1

.. second::
   :opt2: val2
"#;
        let results_first = parse_rst_multiple(rst, &["first"]);
        let opts1 = opts(&[("opt1", "val1")]);
        assert_single_directive_eq_props(&results_first, "first", "", &opts1, "", Some(2));

        let results_second = parse_rst_multiple(rst, &["second"]);
        let opts2 = opts(&[("opt2", "val2")]);
        assert_single_directive_eq_props(&results_second, "second", "", &opts2, "", Some(5));
    }

    #[test]
    fn test_directive_name_is_substring_of_another() {
        let rst = r#"
.. mydirective::
   :k: v

   Content A

.. mydirective-extra::
   :k2: v2

   Content B
"#;
        let results_mydirective = parse_rst_multiple(rst, &["mydirective"]);
        let opts_a = opts(&[("k", "v")]);
        assert_single_directive_eq_props(
            &results_mydirective,
            "mydirective",
            "",
            &opts_a,
            "Content A",
            Some(2),
        );

        let results_extra = parse_rst_multiple(rst, &["mydirective-extra"]);
        let opts_b = opts(&[("k2", "v2")]);
         assert_single_directive_eq_props(
            &results_extra,
            "mydirective-extra",
            "",
            &opts_b,
            "Content B",
            Some(7),
        );
    }
    
    #[test]
    fn test_arbitrary_data_in_option_value() {
        let rst = r#"
.. mydirective::
    :option1: value1
    :option2: value2  // Some other text ..-l. df s...dff; fslkjdjf
    :option3: value3

    Content.
    "#;

        let expected_options = opts(&[
            ("option1", "value1"),
            ("option2", "value2  // Some other text ..-l. df s...dff; fslkjdjf"),
            ("option3", "value3"),
        ]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "Content.",
            Some(2),
        );
    }

    #[test]
    fn test_multiline_option_supported() {
        let rst = r#"
.. mydirective::
    :option1: value1
        second line of value1
    :option2: value2

    Content.
    "#;
        let expected_options = opts(&[
            ("option1", "value1\nsecond line of value1"),
            ("option2", "value2"),
        ]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "Content.",
            Some(2),
        );
    }
    
    #[test]
    fn test_multiline_option_empty_first_line() {
        let rst = r#"
.. mydirective::
    :option1:
        indented line1
        indented line2
    :option2: value2

    Content.
    "#;
         let expected_options = opts(&[
            ("option1", "indented line1\nindented line2"),
            ("option2", "value2"),
        ]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "Content.",
            Some(2),
        );
    }

    #[test]
    fn test_empty_line_within_options_terminates_options() {
        let rst = r#"
    .. mydirective::
       :option1: value1
    
       :option2: value2

       Content.
    "#;
        let expected_options = opts(&[("option1", "value1")]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        // The empty line makes in_options=false. Then ":option2: value2" becomes content.
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            ":option2: value2\n\nContent.", 
            Some(2), // The parser correctly identifies this as line 2
        );
    }

    #[test]
    fn test_empty_line_within_content() {
        let rst = r#"
.. mydirective::
    :option1: value1

    Content line 1.

    Content line 3.
    "#;
        let expected_options = opts(&[("option1", "value1")]); // Corrected: options should be present
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options, // Corrected: use actual expected options
            "Content line 1.\n\nContent line 3.", // Corrected: expected content
            Some(2),
        );
    }
    
    #[test]
    fn test_option_like_line_in_content() {
        let rst = r#"
.. mydirective::
    :real_option: real_value

    This is content.
    This line looks like an option: :fake_option: fake_value
    More content.
"#;
        let expected_options = opts(&[("real_option", "real_value")]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "",
            &expected_options,
            "This is content.\nThis line looks like an option: :fake_option: fake_value\nMore content.", // Corrected: min_indent from content block is applied
            Some(2),
        );
    }

    #[test]
    fn test_directive_with_arguments() {
        let rst = r#"
.. mydirective:: some arguments here
   :option1: value1

   Content.
"#;
        let expected_options = opts(&[("option1", "value1")]);
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "some arguments here",
            &expected_options,
            "Content.",
            Some(2),
        );
    }

    #[test]
    fn test_directive_with_arguments_no_options_no_content() {
        let rst = ".. mydirective:: just arguments\n";
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "just arguments",
            &HashMap::new(),
            "",
            Some(1),
        );
    }

    #[test]
    fn test_directive_with_arguments_no_options() {
        let rst = r#"
.. mydirective:: arguments here

   Content without options.
"#;
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_single_directive_eq_props(
            &results,
            "mydirective",
            "arguments here",
            &HashMap::new(),
            "Content without options.",
            Some(2),
        );
    }

    // Renamed from test_parse_rst_all to reflect it tests parse_rst_multiple
    #[test]
    fn test_parse_rst_multiple_single_target_directive() {
        let rst = r#"
.. mydirective::
   :option1: value1

   Content 1.

Some text in between.

.. mydirective:: arg2
   :option2: value2

   Content 2.
"#;
        // Corrected call to parse_rst_multiple
        let results = parse_rst_multiple(rst, &["mydirective"]); 
        assert_eq!(results.len(), 2);

        let (d1, l1) = &results[0];
        assert_eq!(d1.name, "mydirective");
        assert_eq!(d1.arguments, "");
        assert_eq!(d1.options, opts(&[("option1", "value1")]));
        assert_eq!(d1.content, "Content 1.");
        assert_eq!(*l1, 2); // Line numbers are 1-based

        let (d2, l2) = &results[1];
        assert_eq!(d2.name, "mydirective");
        assert_eq!(d2.arguments, "arg2");
        assert_eq!(d2.options, opts(&[("option2", "value2")]));
        assert_eq!(d2.content, "Content 2.");
        assert_eq!(*l2, 9); // Corrected expected line number
    }

    #[test]
    fn test_parse_rst_multiple_different_directives() {
        let rst = r#"
.. directive1:: D1 Arg
   :opt1: val1

   Content for D1.

.. directive2:: D2 Arg
   :opt2: val2

   Content for D2.

.. directive1:: D1 Arg2
   :opt3: val3

   More content for D1.
"#;
        let results = parse_rst_multiple(rst, &["directive1", "directive2"]);
        assert_eq!(results.len(), 3);

        assert_eq!(results[0].0.name, "directive1");
        assert_eq!(results[0].0.arguments, "D1 Arg");
        assert_eq!(results[0].1, 2);

        assert_eq!(results[1].0.name, "directive2");
        assert_eq!(results[1].0.arguments, "D2 Arg");
        assert_eq!(results[1].1, 7);
        
        assert_eq!(results[2].0.name, "directive1");
        assert_eq!(results[2].0.arguments, "D1 Arg2");
        assert_eq!(results[2].1, 12);
    }

    #[test]
    fn test_parse_rst_multiple_no_matches() {
        let rst = r#"
.. otherdirective::
   Content.
"#;
        let results = parse_rst_multiple(rst, &["mydirective"]);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parse_rst_multiple_empty_input() {
        let results = parse_rst_multiple("", &["mydirective"]);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parse_rst_multiple_tightly_packed() {
        let rst = ".. d1::\n.. d2::\n.. d1::arg"; // Changed to valid syntax with space
        let results = parse_rst_multiple(rst, &["d1", "d2"]);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0.name, "d1");
        assert_eq!(results[0].0.arguments, "");
        assert_eq!(results[0].1, 1);
        assert_eq!(results[1].0.name, "d2");
        assert_eq!(results[1].1, 2);
        assert_eq!(results[2].0.name, "d1");
        assert_eq!(results[2].0.arguments, "arg");
        assert_eq!(results[2].1, 3);
    }
    
    #[test]
    fn test_parse_rst_multiple_directive_name_with_space_before_colon() {
        // According to the spec "directive name must be a single word without spaces"
        // So "my dir" is not a valid directive name.
        let rst = ".. my dir :: args\n   :op:val\n\n   content";
        let results = parse_rst_multiple(rst, &["my dir"]);
        assert_eq!(results.len(), 0); // Expect 0 as "my dir" is invalid
    }

    #[test]
    fn test_parse_rst_multiple_false_starts() {
        let rst = "Some text .. notadirective\n.. realdir::\nText .. also not :: a directive";
        let results = parse_rst_multiple(rst, &["realdir"]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.name, "realdir");
        assert_eq!(results[0].1, 2); // Line number of ".. realdir::"
    }
}

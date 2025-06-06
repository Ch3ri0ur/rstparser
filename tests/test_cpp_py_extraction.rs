use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use rstparser::{FileWalker, Processor};

#[test]
fn test_cpp_file_extraction() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.cpp");
    
    // Create a test C++ file with RST content
    let cpp_content = r#"
/// Some C++ code documentation
///
/// @rst
/// .. mydirective::
///    :option1: value1
///
///    This is RST content in a C++ file.
/// @endrst
///
/// More C++ code
"#;
    
    File::create(&file_path).unwrap().write_all(cpp_content.as_bytes()).unwrap();
    
    // Create processor to find mydirective
    let processor = Processor::new(vec!["mydirective".to_string()]);
    let result = processor.process_file(&file_path).unwrap();
    
    // Should find 1 directive
    assert_eq!(result.len(), 1);
    
    // Check directive name
    assert_eq!(result[0].directive.name, "mydirective");
    
    // Check options
    assert_eq!(result[0].directive.options.get("option1").unwrap(), "value1");
    
    // Check content
    assert_eq!(result[0].directive.content, "This is RST content in a C++ file.");
    
    // Check source file
    assert_eq!(result[0].source_file, file_path.to_string_lossy().to_string());
}

#[test]
fn test_python_file_extraction() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.py");
    
    // Create a test Python file with RST content
    let py_content = r#"
def some_function():
    """
    Some Python docstring
    
    @rst
    .. mydirective::
       :option1: value1
    
       This is RST content in a Python file.
    @endrst
    
    More docstring content
    """
    pass
"#;
    
    File::create(&file_path).unwrap().write_all(py_content.as_bytes()).unwrap();
    
    // Create processor to find mydirective
    let processor = Processor::new(vec!["mydirective".to_string()]);
    let result = processor.process_file(&file_path).unwrap();
    
    // Should find 1 directive
    assert_eq!(result.len(), 1);
    
    // Check directive name
    assert_eq!(result[0].directive.name, "mydirective");
    
    // Check options
    assert_eq!(result[0].directive.options.get("option1").unwrap(), "value1");
    
    // Check content
    assert_eq!(result[0].directive.content, "This is RST content in a Python file.");
    
    // Check source file
    assert_eq!(result[0].source_file, file_path.to_string_lossy().to_string());
}

#[test]
fn test_file_walker_finds_cpp_py_files() {
    // Create a temporary directory structure
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();
    
    // Create some test files
    let file1_path = temp_path.join("file1.rst");
    let file2_path = temp_path.join("file2.cpp");
    let file3_path = temp_path.join("file3.py");
    let file4_path = temp_path.join("file4.txt");
    
    // Create the files
    File::create(&file1_path).unwrap().write_all(b"test content").unwrap();
    File::create(&file2_path).unwrap().write_all(b"test content").unwrap();
    File::create(&file3_path).unwrap().write_all(b"test content").unwrap();
    File::create(&file4_path).unwrap().write_all(b"test content").unwrap();
    
    // Test with default settings (should find .rst, .cpp, and .py files)
    let walker = FileWalker::new();
    let files = walker.find_files(temp_path).unwrap();
    
    // Should find 3 files (.rst, .cpp, .py)
    assert_eq!(files.len(), 3);
    assert!(files.contains(&file1_path));
    assert!(files.contains(&file2_path));
    assert!(files.contains(&file3_path));
    assert!(!files.contains(&file4_path));
}

#[test]
fn test_multiple_rst_blocks_in_cpp() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.cpp");
    
    // Create a test C++ file with multiple RST blocks
    let cpp_content = r#"
/// @rst
/// .. directive1::
///    :option1: value1
///
///    Content for directive1.
/// @endrst
///
/// Some code
///
/// @rst
/// .. directive2::
///    :option2: value2
///
///    Content for directive2.
/// @endrst
"#;
    
    File::create(&file_path).unwrap().write_all(cpp_content.as_bytes()).unwrap();
    
    // Create processor to find both directives
    let processor = Processor::new(vec!["directive1".to_string(), "directive2".to_string()]);
    let result = processor.process_file(&file_path).unwrap();
    
    // Should find 2 directives
    assert_eq!(result.len(), 2);
    
    // Check directive names
    assert_eq!(result[0].directive.name, "directive1");
    assert_eq!(result[1].directive.name, "directive2");
    
    // Check options
    assert_eq!(result[0].directive.options.get("option1").unwrap(), "value1");
    assert_eq!(result[1].directive.options.get("option2").unwrap(), "value2");
    
    // Check content
    assert_eq!(result[0].directive.content, "Content for directive1.");
    assert_eq!(result[1].directive.content, "Content for directive2.");
}

#[test]
fn test_multiple_rst_blocks_in_python() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.py");
    
    // Create a test Python file with multiple RST blocks
    let py_content = r#"
"""
@rst
.. directive1::
   :option1: value1

   Content for directive1.
@endrst
"""

def some_function():
    """
    @rst
    .. directive2::
       :option2: value2
    
       Content for directive2.
    @endrst
    """
    pass
"#;
    
    File::create(&file_path).unwrap().write_all(py_content.as_bytes()).unwrap();
    
    // Create processor to find both directives
    let processor = Processor::new(vec!["directive1".to_string(), "directive2".to_string()]);
    let result = processor.process_file(&file_path).unwrap();
    
    // Should find 2 directives
    assert_eq!(result.len(), 2);
    
    // Check directive names
    assert_eq!(result[0].directive.name, "directive1");
    assert_eq!(result[1].directive.name, "directive2");
    
    // Check options
    assert_eq!(result[0].directive.options.get("option1").unwrap(), "value1");
    assert_eq!(result[1].directive.options.get("option2").unwrap(), "value2");
    
    // Check content
    assert_eq!(result[0].directive.content, "Content for directive1.");
    assert_eq!(result[1].directive.content, "Content for directive2.");
}

#[test]
fn test_multiline_option_as_last_option_in_cpp() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_multiline_option.cpp");
    
    // Create a test C++ file with a multiline option as the last option
    // Using a different format for the multiline option
    let cpp_content = r#"
/// @rst
/// .. mydirective::
///    :option1: value1
///    :option2: indented line1
///             indented line2
///
///    Content after multiline option.
/// @endrst
"#;
    
    // Print the raw content for debugging
    println!("Original C++ content: {:?}", cpp_content);
    
    File::create(&file_path).unwrap().write_all(cpp_content.as_bytes()).unwrap();
    
    // Create processor to find the directive
    let processor = Processor::new(vec!["mydirective".to_string()]);
    let result = processor.process_file(&file_path).unwrap();
    
    // Should find 1 directive
    assert_eq!(result.len(), 1);
    
    // Check directive name
    assert_eq!(result[0].directive.name, "mydirective");
    
    // Debug output
    println!("Options: {:?}", result[0].directive.options);
    println!("Content: {:?}", result[0].directive.content);
    
    // Extract the raw RST content to debug
    let raw_content = rstparser::extractor::RstExtractor::extract_from_cpp(cpp_content);
    println!("Raw extracted content: {:?}", raw_content);
    
    // Extract the raw RST content
    let raw_content = rstparser::extractor::RstExtractor::extract_from_cpp(cpp_content);
    
    // Manually parse the raw content to debug the issue
    let parsed_results_vec = rstparser::parser::parse_rst_multiple(&raw_content, &["mydirective"]);
    println!("Manually parsed options: {:?}", parsed_results_vec.first().map(|(d, _)| &d.options));
    
    // Check options
    assert_eq!(result[0].directive.options.get("option1").unwrap(), "value1");
    assert_eq!(result[0].directive.options.get("option2").unwrap(), "indented line1\nindented line2");
    
    // Check content
    assert_eq!(result[0].directive.content, "Content after multiline option.");
}

#[test]
fn test_multiline_option_as_last_option_in_python() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_multiline_option.py");
    
    // Create a test Python file with a multiline option as the last option
    let py_content = r#"
def some_function():
    """
    @rst
    .. mydirective::
       :option1: value1
       :option2:
           indented line1
           indented line2
           
       Content after multiline option.
    @endrst
    """
    pass
"#;
    
    File::create(&file_path).unwrap().write_all(py_content.as_bytes()).unwrap();
    
    // Create processor to find the directive
    let processor = Processor::new(vec!["mydirective".to_string()]);
    let result = processor.process_file(&file_path).unwrap();
    
    // Should find 1 directive
    assert_eq!(result.len(), 1);
    
    // Check directive name
    assert_eq!(result[0].directive.name, "mydirective");
    
    // Debug output
    println!("Python Options: {:?}", result[0].directive.options);
    println!("Python Content: {:?}", result[0].directive.content);
    
    // Check options
    assert_eq!(result[0].directive.options.get("option1").unwrap(), "value1");
    assert_eq!(result[0].directive.options.get("option2").unwrap(), "indented line1\nindented line2");
    
    // Check content
    assert_eq!(result[0].directive.content, "Content after multiline option.");
}

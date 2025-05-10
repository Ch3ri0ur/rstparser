use rstparser::parser::parse_rst_multiple; // Removed unused parse_rst
use rstparser::timing::Timer;
use rstparser::time_it;
use rstparser::time_call;
use std::fs;
use std::path::PathBuf;
use std::error::Error;

// Helper function to create a test RST file with specified content
fn create_test_file(filename: &str, content: &str) -> PathBuf {
    let file_path = PathBuf::from(filename);
    fs::write(&file_path, content).unwrap();
    file_path
}

// Helper function to create RST content with a single directive
fn create_rst_with_single_directive(directive_name: &str, content_size: usize) -> String {
    let mut rst = format!(".. {}::\n", directive_name);
    rst.push_str("   :option1: value1\n");
    rst.push_str("   :option2: value2\n\n");
    
    // Add content of specified size
    for i in 0..content_size {
        rst.push_str(&format!("   Line {} of content.\n", i));
    }
    
    rst
}

// Helper function to create RST content with multiple directives
fn create_rst_with_multiple_directives(directive_name: &str, directive_count: usize, content_size: usize) -> String {
    let mut rst = String::new();
    
    for i in 0..directive_count {
        rst.push_str(&format!(".. {}::\n", directive_name));
        rst.push_str(&format!("   :option{}: value{}\n\n", i, i));
        
        // Add content of specified size
        for j in 0..content_size {
            rst.push_str(&format!("   Line {} of content for directive {}.\n", j, i));
        }
        
        // Add some text between directives
        rst.push_str("\nSome text between directives.\n\n");
    }
    
    rst
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("RST Parser Timing Example");
    println!("========================\n");
    
    // Example 1: Time parsing a single directive with different content sizes
    println!("Example 1: Parsing a single directive with different content sizes");
    println!("--------------------------------------------------------------");
    
    for content_size in [10, 100, 1000, 10000] {
        let rst = create_rst_with_single_directive("mydirective", content_size);
        
        // Using the time_call macro
        let _directive = time_call!(&format!("parse_rst_multiple (single, content_size={})", content_size), parse_rst_multiple, &rst, &["mydirective"]);
    }
    println!();
    
    // Example 2: Time parsing multiple directives of the same type
    println!("Example 2: Parsing multiple directives of the same type");
    println!("----------------------------------------------------");
    
    for directive_count in [10, 50, 100, 500] {
        let rst = create_rst_with_multiple_directives("mydirective", directive_count, 10);
        
        // Using the time_it macro
        let _directives = time_it!(&format!("parse_rst_multiple (directive_count={})", directive_count), {
            parse_rst_multiple(&rst, &["mydirective"])
        });
    }
    println!();
    
    // Example 3: Time parsing multiple different directives
    println!("Example 3: Parsing multiple different directives");
    println!("---------------------------------------------");
    
    let directive_names = ["directive1", "directive2", "directive3", "directive4", "directive5"];
    
    for &count in [1, 2, 3, 4, 5].iter() {
        let names = &directive_names[0..count];
        
        // Create content with one of each directive type
        let mut rst = String::new();
        for &name in names {
            rst.push_str(&create_rst_with_single_directive(name, 10));
            rst.push_str("\n\n");
        }
        
        // Using a Timer directly
        let timer = Timer::new(&format!("parse_rst_multiple (directive_types={})", count));
        let _directives = parse_rst_multiple(&rst, names);
        timer.report();
    }
    println!();
    
    // Example 4: Time the entire process with a large file
    println!("Example 4: End-to-end timing with a large file");
    println!("-------------------------------------------");
    
    // Create a large RST file with many directives
    let large_rst = create_rst_with_multiple_directives("mydirective", 1000, 10);
    let file_path = create_test_file("large_test.rst", &large_rst);
    
    // Time reading the file
    let timer = Timer::new("Read file");
    let content = fs::read_to_string(&file_path)?;
    timer.report();
    
    // Time parsing all directives
    let timer = Timer::new("Parse all directives");
    let directives = parse_rst_multiple(&content, &["mydirective"]);
    timer.report();
    
    println!("Found {} directives", directives.len());
    
    // Clean up
    // fs::remove_file(file_path)?;
    
    Ok(())
}

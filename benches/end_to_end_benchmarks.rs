use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rstparser::file_walker::FileWalker;
use rstparser::processor::Processor;
use rstparser::aggregator::{Aggregator, GroupBy};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

// Helper function to create a test RST file with specified content
fn create_test_file(dir_path: &PathBuf, filename: &str, content: &str) -> PathBuf {
    let file_path = dir_path.join(filename);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file_path
}

// Helper function to create RST content with multiple directives
fn create_rst_with_directives(directive_names: &[&str], directives_per_name: usize, content_size: usize) -> String {
    let mut rst = String::new();
    
    for &name in directive_names {
        for i in 0..directives_per_name {
            rst.push_str(&format!(".. {}::\n", name));
            rst.push_str(&format!("   :option{}: value{}\n\n", i, i));
            
            // Add content of specified size
            for j in 0..content_size {
                rst.push_str(&format!("   Line {} of content for directive {} instance {}.\n", j, name, i));
            }
            
            // Add some text between directives
            rst.push_str("\nSome text between directives.\n\n");
        }
    }
    
    rst
}

// Helper function to create a directory structure with test files
fn create_test_directory_structure(
    root_dir: &PathBuf,
    depth: usize,
    files_per_dir: usize,
    directive_names: &[&str],
    directives_per_name: usize,
    content_size: usize,
) -> usize {
    let mut total_files = 0;
    
    // Create files in the current directory
    for i in 0..files_per_dir {
        let content = create_rst_with_directives(directive_names, directives_per_name, content_size);
        let file_path = create_test_file(root_dir, &format!("file_{}.rst", i), &content);
        total_files += 1;
    }
    
    // Create subdirectories if depth > 0
    if depth > 0 {
        for i in 0..3 { // Create 3 subdirectories at each level
            let subdir_path = root_dir.join(format!("subdir_{}", i));
            fs::create_dir_all(&subdir_path).unwrap();
            
            // Recursively create files in subdirectories
            total_files += create_test_directory_structure(
                &subdir_path,
                depth - 1,
                files_per_dir,
                directive_names,
                directives_per_name,
                content_size,
            );
        }
    }
    
    total_files
}

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    
    // Create a temporary directory for test files and output
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create test directory structures with different characteristics
    let test_configs = [
        // (depth, files_per_dir, directives_per_name, content_size, name)
        (1, 5, 2, 5, "small"),
        (2, 5, 5, 10, "medium"),
        (3, 5, 10, 20, "large"),
    ];
    
    let directive_names = ["directive1", "directive2"];
    
    for &(depth, files_per_dir, directives_per_name, content_size, name) in &test_configs {
        // Create test directory structure
        let test_dir_path = temp_path.join(name);
        fs::create_dir_all(&test_dir_path).unwrap();
        
        let total_files = create_test_directory_structure(
            &test_dir_path,
            depth,
            files_per_dir,
            &directive_names,
            directives_per_name,
            content_size,
        );
        
        println!("Created {} files for {} test", total_files, name);
        
        // Create output directory
        let output_dir = temp_path.join(format!("{}_output", name));
        
        // Benchmark the end-to-end process
        group.bench_function(BenchmarkId::new("pipeline", name), |b| {
            b.iter(|| {
                // Step 1: Find RST files
                let walker = FileWalker::new();
                let files = walker.find_files(black_box(&test_dir_path)).unwrap();
                
                // Step 2: Process files to find directives
                let processor = Processor::new(directive_names.iter().map(|&s| s.to_string()).collect());
                let directives = processor.process_files(black_box(files)).unwrap();
                
                // Step 3: Aggregate directives to JSON files
                let aggregator = Aggregator::new(&output_dir, GroupBy::DirectiveName);
                aggregator.aggregate_to_json(black_box(directives)).unwrap()
            })
        });
    }
    
    group.finish();
}

criterion_group!(end_to_end_benches, bench_end_to_end);
criterion_main!(end_to_end_benches);

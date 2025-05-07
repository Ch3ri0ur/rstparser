use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rstparser::processor::Processor;
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

fn bench_process_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_file");
    
    // Create a temporary directory for test files
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create test files with different numbers of directives
    let directive_names = ["directive1", "directive2"];
    
    for directives_per_name in [5, 20, 50].iter() {
        let content = create_rst_with_directives(&directive_names, *directives_per_name, 5);
        let file_path = create_test_file(&temp_path, &format!("test_{}.rst", directives_per_name), &content);
        
        // Create processor to find the directives
        let processor = Processor::new(directive_names.iter().map(|&s| s.to_string()).collect());
        
        group.bench_with_input(
            BenchmarkId::new("directives_per_name", directives_per_name), 
            &file_path,
            |b, file_path| {
                b.iter(|| processor.process_file(black_box(file_path)))
            }
        );
    }
    
    group.finish();
}

fn bench_process_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_files");
    
    // Create a temporary directory for test files
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create test files with different numbers of directives
    let directive_names = ["directive1", "directive2"];
    let content = create_rst_with_directives(&directive_names, 10, 5);
    
    // Create different numbers of files
    for file_count in [5, 20, 50].iter() {
        let mut file_paths = Vec::new();
        
        for i in 0..*file_count {
            let file_path = create_test_file(&temp_path, &format!("test_{}_{}.rst", file_count, i), &content);
            file_paths.push(file_path);
        }
        
        // Create processor to find the directives
        let processor = Processor::new(directive_names.iter().map(|&s| s.to_string()).collect());
        
        group.bench_with_input(
            BenchmarkId::new("file_count", file_count), 
            &file_paths,
            |b, file_paths| {
                b.iter(|| processor.process_files(black_box(file_paths.clone())))
            }
        );
    }
    
    group.finish();
}

criterion_group!(processor_benches, bench_process_file, bench_process_files);
criterion_main!(processor_benches);

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rstparser::file_walker::FileWalker;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

// Helper function to create a directory structure with test files
fn create_test_directory_structure(
    root_dir: &PathBuf,
    depth: usize,
    files_per_dir: usize,
    extensions: &[&str],
) -> usize {
    let mut total_files = 0;
    
    // Create files in the current directory
    for ext in extensions {
        for i in 0..files_per_dir {
            let file_path = root_dir.join(format!("file_{}_{}.{}", i, ext, ext));
            File::create(&file_path).unwrap().write_all(b"test content").unwrap();
            total_files += 1;
        }
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
                extensions,
            );
        }
    }
    
    total_files
}

fn bench_find_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_files");
    
    // Create a temporary directory for test files
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Test with different directory depths
    for depth in [1, 2, 3].iter() {
        // Create a directory structure with test files
        let extensions = ["rst", "txt", "md"];
        let files_per_dir = 5;
        
        let subdir_path = temp_path.join(format!("depth_{}", depth));
        fs::create_dir_all(&subdir_path).unwrap();
        
        let total_files = create_test_directory_structure(
            &subdir_path,
            *depth,
            files_per_dir,
            &extensions,
        );
        
        println!("Created {} files for depth {}", total_files, depth);
        
        // Benchmark finding .rst files
        let walker = FileWalker::new().with_extensions(vec!["rst".to_string()]);
        
        group.bench_with_input(
            BenchmarkId::new("depth", depth), 
            &subdir_path,
            |b, dir_path| {
                b.iter(|| walker.find_files(black_box(dir_path)))
            }
        );
    }
    
    // Test with different file extensions
    let extensions_to_test = [
        vec!["rst".to_string()],
        vec!["rst".to_string(), "txt".to_string()],
        vec!["rst".to_string(), "txt".to_string(), "md".to_string()],
    ];
    
    let flat_dir_path = temp_path.join("extensions_test");
    fs::create_dir_all(&flat_dir_path).unwrap();
    
    // Create files with different extensions
    let extensions = ["rst", "txt", "md", "html", "css"];
    let files_per_ext = 10;
    
    for ext in &extensions {
        for i in 0..files_per_ext {
            let file_path = flat_dir_path.join(format!("file_{}_{}.{}", i, ext, ext));
            File::create(&file_path).unwrap().write_all(b"test content").unwrap();
        }
    }
    
    // Benchmark finding files with different extension combinations
    for (i, exts) in extensions_to_test.iter().enumerate() {
        let walker = FileWalker::new().with_extensions(exts.clone());
        
        group.bench_with_input(
            BenchmarkId::new("extensions", i + 1), 
            &flat_dir_path,
            |b, dir_path| {
                b.iter(|| walker.find_files(black_box(dir_path)))
            }
        );
    }
    
    group.finish();
}

fn bench_find_files_with_max_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_files_with_max_depth");
    
    // Create a temporary directory for test files
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create a deep directory structure
    let depth = 5;
    let extensions = ["rst", "txt"];
    let files_per_dir = 3;
    
    let deep_dir_path = temp_path.join("deep_structure");
    fs::create_dir_all(&deep_dir_path).unwrap();
    
    let total_files = create_test_directory_structure(
        &deep_dir_path,
        depth,
        files_per_dir,
        &extensions,
    );
    
    println!("Created {} files for max_depth tests", total_files);
    
    // Benchmark with different max_depth values
    for max_depth in [1, 2, 3, 4, 5].iter() {
        let walker = FileWalker::new()
            .with_extensions(vec!["rst".to_string()])
            .with_max_depth(*max_depth);
        
        group.bench_with_input(
            BenchmarkId::new("max_depth", max_depth), 
            &deep_dir_path,
            |b, dir_path| {
                b.iter(|| walker.find_files(black_box(dir_path)))
            }
        );
    }
    
    group.finish();
}

criterion_group!(file_walker_benches, bench_find_files, bench_find_files_with_max_depth);
criterion_main!(file_walker_benches);

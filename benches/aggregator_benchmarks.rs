use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rstparser::aggregator::{Aggregator, DirectiveWithSource, GroupBy};
use rstparser::parser::Directive;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

// Helper function to create a test directive
fn create_test_directive(name: &str, index: usize, options_count: usize, content_size: usize) -> Directive {
    let mut options = HashMap::new();
    
    for i in 0..options_count {
        options.insert(format!("option{}", i), format!("value{}", i));
    }
    
    let mut content = String::new();
    for i in 0..content_size {
        content.push_str(&format!("Line {} of content for directive {} instance {}.\n", i, name, index));
    }
    
    Directive {
        name: name.to_string(),
        arguments: format!("arg{}", index),
        options,
        content,
    }
}

// Helper function to create a vector of test directives with source information
fn create_test_directives_with_source(
    directive_names: &[&str],
    directives_per_name: usize,
    options_count: usize,
    content_size: usize,
    source_files: &[&str],
) -> Vec<DirectiveWithSource> {
    let mut directives = Vec::new();
    
    for &name in directive_names {
        for i in 0..directives_per_name {
            let directive = create_test_directive(name, i, options_count, content_size);
            
            // Assign to a source file (round-robin)
            let source_file = source_files[i % source_files.len()];
            
            directives.push(DirectiveWithSource {
                directive,
                source_file: source_file.to_string(),
                line_number: Some(i * 10), // Arbitrary line number
            });
        }
    }
    
    directives
}

fn bench_aggregate_to_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_to_json");
    
    // Create a temporary directory for output
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().to_path_buf();
    
    // Test with different grouping strategies
    let directive_names = ["directive1", "directive2", "directive3"];
    let source_files = ["file1.rst", "file2.rst", "file3.rst", "file4.rst", "file5.rst"];
    
    // Create test directives
    let directives_small = create_test_directives_with_source(
        &directive_names[0..2],
        10,
        3,
        5,
        &source_files[0..2],
    );
    
    let directives_medium = create_test_directives_with_source(
        &directive_names,
        20,
        5,
        10,
        &source_files,
    );
    
    let directives_large = create_test_directives_with_source(
        &directive_names,
        50,
        10,
        20,
        &source_files,
    );
    
    // Benchmark different grouping strategies with small dataset
    for group_by in [GroupBy::DirectiveName, GroupBy::All, GroupBy::SourceFile].iter() {
        let output_subdir = output_path.join(format!("small_{:?}", group_by));
        let aggregator = Aggregator::new(&output_subdir, *group_by);
        
        group.bench_with_input(
            BenchmarkId::new("small", format!("{:?}", group_by)), 
            &directives_small,
            |b, directives| {
                b.iter(|| aggregator.aggregate_to_json(black_box(directives.clone())))
            }
        );
    }
    
    // Benchmark different grouping strategies with medium dataset
    for group_by in [GroupBy::DirectiveName, GroupBy::All, GroupBy::SourceFile].iter() {
        let output_subdir = output_path.join(format!("medium_{:?}", group_by));
        let aggregator = Aggregator::new(&output_subdir, *group_by);
        
        group.bench_with_input(
            BenchmarkId::new("medium", format!("{:?}", group_by)), 
            &directives_medium,
            |b, directives| {
                b.iter(|| aggregator.aggregate_to_json(black_box(directives.clone())))
            }
        );
    }
    
    // Benchmark different grouping strategies with large dataset
    for group_by in [GroupBy::DirectiveName, GroupBy::All, GroupBy::SourceFile].iter() {
        let output_subdir = output_path.join(format!("large_{:?}", group_by));
        let aggregator = Aggregator::new(&output_subdir, *group_by);
        
        group.bench_with_input(
            BenchmarkId::new("large", format!("{:?}", group_by)), 
            &directives_large,
            |b, directives| {
                b.iter(|| aggregator.aggregate_to_json(black_box(directives.clone())))
            }
        );
    }
    
    group.finish();
}

criterion_group!(aggregator_benches, bench_aggregate_to_json);
criterion_main!(aggregator_benches);

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rstparser::parser::parse_rst_multiple; // Removed unused parse_rst
use std::collections::HashMap;

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

// Helper function to create RST content with multiple different directives
fn create_rst_with_multiple_different_directives(
    directive_names: &[&str],
    content_size: usize,
) -> String {
    let mut rst = String::new();

    for (i, &name) in directive_names.iter().enumerate() {
        rst.push_str(&format!(".. {}::\n", name));
        rst.push_str(&format!("   :option{}: value{}\n\n", i, i));

        // Add content of specified size
        for j in 0..content_size {
            rst.push_str(&format!(
                "   Line {} of content for directive {}.\n",
                j, name
            ));
        }

        // Add some text between directives
        rst.push_str("\nSome text between directives.\n\n");
    }

    rst
}

// Helper function to create RST content with multiple instances of each specified directive type
fn create_rst_with_multiple_instances_of_directives(
    directive_names: &[&str], // The unique directive types
    instances_per_name: usize, // How many times each name is repeated
    content_size: usize,
) -> String {
    let mut rst = String::new();
    for &name in directive_names.iter() { // For each unique directive type
        for _instance_idx in 0..instances_per_name { // Create 'instances_per_name' instances
            rst.push_str(&format!(".. {}::\n", name)); // Use the same directive name
            rst.push_str("   :common_option1: valueA\n"); // Generic options
            rst.push_str("   :common_option2: valueB\n\n");

            for j in 0..content_size {
                rst.push_str(&format!(
                    "   Line {} of content for directive {}.\n", // Content can be generic too
                    j, name
                ));
            }
            rst.push_str("\nSome text between directives.\n\n"); // Separator
        }
    }
    rst
}

fn bench_parse_rst(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_rst");

    // Benchmark parsing a single directive with different content sizes
    for content_size in [10, 100, 1000].iter() {
        let rst = create_rst_with_single_directive("mydirective", *content_size);

        group.bench_with_input(
            BenchmarkId::new("content_size", content_size),
            &rst,
            |b, rst| b.iter(|| parse_rst_multiple(black_box(rst), black_box(&["mydirective"]))), // Changed to parse_rst_multiple
        );
    }

    group.finish();
}

fn bench_parse_rst_multiple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_rst_multiple_instances");

    // Generate a list of unique directive names
    // Using up to 20 unique directives
    let max_directive_types = 20; 
    let all_directive_names_strings: Vec<String> = (1..=max_directive_types)
        .map(|i| format!("directive{}", i))
        .collect();
    let all_directive_names_refs: Vec<&str> = all_directive_names_strings
        .iter()
        .map(String::as_str)
        .collect();

    // Define the parameters for the benchmarks
    // Content lines per directive instance is fixed at 10
    let content_lines_per_directive = 10; 
    // Number of different directive types to include
    let num_directive_types_values = [1, 5, 10, 20]; 
    // Number of instances for each directive type
    let num_instances_per_type_values = [1, 10, 25, 50]; 

    for &instances_per_type in num_instances_per_type_values.iter() {
        for &num_types in num_directive_types_values.iter() {
            // Ensure num_types does not exceed the available unique directive names
            if num_types > all_directive_names_refs.len() {
                continue;
            }

            let current_directive_names_slice = &all_directive_names_refs[0..num_types];
            
            let rst_content = create_rst_with_multiple_instances_of_directives(
                current_directive_names_slice,
                instances_per_type,
                content_lines_per_directive,
            );

            let series_name = format!("instances_per_type_{}", instances_per_type);
            
            group.bench_with_input(
                BenchmarkId::new(series_name, num_types), // Parameter for this series is num_types
                &rst_content,
                |b, rst| {
                    b.iter(|| {
                        parse_rst_multiple(black_box(rst), black_box(current_directive_names_slice))
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(parser_benches, bench_parse_rst, bench_parse_rst_multiple);
criterion_main!(parser_benches);

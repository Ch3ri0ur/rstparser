use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rstparser::parser::{parse_rst, parse_rst_multiple};
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

fn bench_parse_rst(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_rst");

    // Benchmark parsing a single directive with different content sizes
    for content_size in [10, 100, 1000].iter() {
        let rst = create_rst_with_single_directive("mydirective", *content_size);

        group.bench_with_input(
            BenchmarkId::new("content_size", content_size),
            &rst,
            |b, rst| b.iter(|| parse_rst(black_box(rst), black_box("mydirective"))),
        );
    }

    group.finish();
}

fn bench_parse_rst_multiple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_rst_multiple");

    // Benchmark parsing multiple different directives
    let directive_names = [
        "directive1",
        "directive2",
        "directive3",
        "directive4",
        "directive5",
        "directive6",
        "directive7",
        "directive8",
        "directive9",
        "directive10",
    ];
    for directive_count in [100, 1000, 100000].iter() {
        for &count in [1,2, 10].iter() {
            let names = &directive_names[0..count];
            let rst = create_rst_with_multiple_different_directives(names, *directive_count);

            group.bench_with_input(
                BenchmarkId::new("directive_types", count),
                &rst,
                |b, rst| b.iter(|| parse_rst_multiple(black_box(rst), black_box(names))),
            );
        }
    }

    group.finish();
}

criterion_group!(parser_benches, bench_parse_rst, bench_parse_rst_multiple);
criterion_main!(parser_benches);

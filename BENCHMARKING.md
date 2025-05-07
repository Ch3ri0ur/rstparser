# Benchmarking the RST Directive Parser

This document provides information on how to benchmark the RST directive parser to measure its performance and identify potential bottlenecks.

## Benchmark Framework

The benchmarking framework uses [Criterion](https://github.com/bheisler/criterion.rs), a statistics-driven benchmarking library for Rust. Criterion provides detailed statistical analysis of benchmark results, including mean, median, and standard deviation of execution times, as well as HTML reports with graphs.

## Benchmark Components

The benchmarking framework includes benchmarks for the following components:

1. **Parser Benchmarks** (`parser_benchmarks.rs`): Benchmarks the core parsing functions:
   - `parse_rst`: Parsing a single directive
   - `parse_rst_all`: Parsing multiple directives of the same type
   - `parse_rst_multiple`: Parsing multiple different directives

2. **Processor Benchmarks** (`processor_benchmarks.rs`): Benchmarks the file processing functions:
   - `process_file`: Processing a single file
   - `process_files`: Processing multiple files in parallel

3. **File Walker Benchmarks** (`file_walker_benchmarks.rs`): Benchmarks the file discovery functions:
   - `find_files`: Finding files with specific extensions
   - `find_files_with_max_depth`: Finding files with a maximum directory depth

4. **Aggregator Benchmarks** (`aggregator_benchmarks.rs`): Benchmarks the directive aggregation functions:
   - `aggregate_to_json`: Aggregating directives to JSON files with different grouping strategies

5. **End-to-End Benchmarks** (`end_to_end_benchmarks.rs`): Benchmarks the entire pipeline from file discovery to JSON output.

## Running Benchmarks

To run all benchmarks, use the provided script:

```bash
./run_benchmarks.sh
```

This will run all benchmarks and generate HTML reports in the `target/criterion` directory.

To run a specific benchmark, use:

```bash
cargo bench --bench <benchmark_name>
```

For example:

```bash
cargo bench --bench parser_benchmarks
```

## Benchmark Results

Benchmark results are stored in the `target/criterion` directory. You can view the HTML reports by opening `target/criterion/report/index.html` in a web browser.

The reports include:

- Mean, median, and standard deviation of execution times
- Graphs showing the distribution of execution times
- Comparison with previous benchmark runs (if available)

## Profiling

For more detailed performance analysis, you can use profiling tools like `perf` or `flamegraph` to identify bottlenecks in the code.

### Using perf

```bash
cargo build --release --bench <benchmark_name>
perf record -g target/release/deps/<benchmark_name>
perf report
```

### Using flamegraph

```bash
cargo flamegraph --bench <benchmark_name>
```

This will generate a flamegraph SVG file that you can open in a web browser to visualize the call stack and identify bottlenecks.

## Optimization Opportunities

Based on the benchmark results, you may identify optimization opportunities in the following areas:

1. **Parser Efficiency**:
   - Improve string handling
   - Optimize regex patterns or replace with manual parsing
   - Reduce memory allocations

2. **Parallel Processing**:
   - Fine-tune the parallelism strategy
   - Optimize work distribution

3. **Memory Usage**:
   - Reduce cloning of data
   - Use more efficient data structures

## Benchmark Parameters

The benchmarks use various parameters to test different scenarios:

- **Content Size**: The size of directive content (number of lines)
- **Directive Count**: The number of directives in a file
- **File Count**: The number of files to process
- **Directory Depth**: The depth of the directory structure
- **Grouping Strategy**: How directives are grouped in output files

You can modify these parameters in the benchmark files to test different scenarios.

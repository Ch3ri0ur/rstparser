#!/bin/bash

# Script to run all benchmarks and generate a report

# Create a directory for benchmark results
mkdir -p benchmark_results

# Run all benchmarks
echo "Running parser benchmarks..."
cargo bench --bench parser_benchmarks

echo "Running processor benchmarks..."
cargo bench --bench processor_benchmarks

echo "Running file_walker benchmarks..."
cargo bench --bench file_walker_benchmarks

echo "Running aggregator benchmarks..."
cargo bench --bench aggregator_benchmarks

echo "Running end-to-end benchmarks..."
cargo bench --bench end_to_end_benchmarks

echo "All benchmarks completed!"
echo "Benchmark results are available in the target/criterion directory."
echo "You can view the HTML reports by opening target/criterion/report/index.html in a web browser."

# Tech Context

## Technologies Used

- Rust (Edition 2024, primary language for the parser library and CLI)
- Shell scripting (for `run_benchmarks.sh`, `run_timing_example.sh`)

## Development Setup

- Rust development environment (Cargo for build and dependency management).
- A shell environment for running benchmark and example scripts.

## Technical Constraints

- Performance is a key consideration, indicated by the extensive benchmarking setup.
- Compatibility with standard RST syntax.
- Extensibility for custom directives and roles.

## Dependencies

- `walkdir = "2.4.0"`: For recursively iterating over directories.
- `serde = { version = "1.0", features = ["derive"] }`: For serializing and deserializing data structures.
- `serde_json = "1.0"`: For working with JSON data.
- `rayon = "1.8.0"`: For data parallelism.
- `clap = { version = "4.4", features = ["derive"] }`: For parsing command-line arguments.

### Dev Dependencies
- `tempfile = "3.8.0"`: For creating temporary files and directories in tests/benchmarks.
- `criterion = "0.5.1"`: For benchmarking.

## Tool Usage Patterns

- `cargo build` / `cargo run` for Rust components.
- `cargo test` for Rust unit/integration tests.
- `cargo bench` for running Rust benchmarks (specific benchmarks defined in `Cargo.toml`: `parser_benchmarks`, `processor_benchmarks`, `file_walker_benchmarks`, `aggregator_benchmarks`, `end_to_end_benchmarks`, `extractor_benchmarks`).
- Shell scripts (`.sh`) for automating benchmark runs and examples.

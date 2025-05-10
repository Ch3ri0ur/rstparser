# Tech Context

## Technologies Used

- Rust (primary language for the parser library and CLI)
- Shell scripting (for `run_benchmarks.sh`, `run_timing_example.sh`)

## Development Setup

- Rust development environment (Cargo for build and dependency management).
- A shell environment for running benchmark and example scripts.

## Technical Constraints

- Performance is a key consideration, indicated by the extensive benchmarking setup.
- Compatibility with standard RST syntax.
- Extensibility for custom directives and roles.

## Dependencies

- `walkdir`: For recursively iterating over directories.
- `serde` (with `derive` feature): For serializing and deserializing data structures.
- `serde_json`: For working with JSON data.
- `rayon`: For data parallelism.
- `clap` (with `derive` feature): For parsing command-line arguments.

### Dev Dependencies
- `tempfile`: For creating temporary files and directories in tests/benchmarks.
- `criterion`: For benchmarking.

## Tool Usage Patterns

- `cargo build` / `cargo run` for Rust components.
- `cargo test` for Rust unit/integration tests.
- `cargo bench` for running Rust benchmarks.
- Shell scripts (`.sh`) for automating benchmark runs and examples.
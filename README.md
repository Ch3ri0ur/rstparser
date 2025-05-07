# RST Directive Parser

A Rust library for parsing reStructuredText (RST) directives.

## Features

- Parse RST directives from text
- Find files with specific extensions
- Process files to extract directives
- Aggregate directives into JSON files
- Parallel processing for improved performance

## Usage

### Basic Usage

```rust
use rstparser::parser::parse_rst;

let rst = r#"
.. mydirective::
   :option1: value1
   :option2: value2

   This is content.
"#;

if let Some(directive) = parse_rst(rst, "mydirective") {
    println!("Found directive: {}", directive.name);
    println!("Options: {:?}", directive.options);
    println!("Content: {}", directive.content);
}
```

### Command Line Usage

```bash
# Find RST files and extract directives
rstparser --dir /path/to/docs --directives note,warning,tip --output output_dir
```

## Testing, Timing, and Benchmarking

This project includes comprehensive tools for testing, timing, and benchmarking the RST directive parser.

### Benchmarking

The benchmarking framework uses [Criterion](https://github.com/bheisler/criterion.rs), a statistics-driven benchmarking library for Rust.

To run all benchmarks:

```bash
./run_benchmarks.sh
```

For more information on benchmarking, see [BENCHMARKING.md](BENCHMARKING.md).

### Timing

The project includes a simple timing utility for measuring execution time:

```rust
use rstparser::timing::Timer;
use rstparser::time_it;
use rstparser::time_call;

// Using a Timer directly
let timer = Timer::new("My operation");
// ... perform operation ...
timer.report();

// Using the time_it macro
let result = time_it!("My operation", {
    // ... perform operation ...
});

// Using the time_call macro
let result = time_call!("My operation", my_function, arg1, arg2);
```

To run the timing example:

```bash
./run_timing_example.sh
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

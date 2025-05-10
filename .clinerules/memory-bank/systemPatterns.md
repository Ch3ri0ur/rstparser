# System Patterns

## System Architecture

The system is designed as a command-line application with a supporting library. The core functionality revolves around parsing reStructuredText (RST) directives from files.

The CLI application (`src/main.rs`) orchestrates the process:
1.  **Argument Parsing**: Uses `clap` to parse command-line arguments (input directory, file extensions, directives to find, output directory, grouping options, max depth).
2.  **File Discovery**: Employs a `FileWalker` module to recursively find files matching specified extensions and depth within a given directory.
3.  **Content Extraction**: An `Extractor` module is responsible for identifying and extracting RST content blocks from various file types (e.g., `.rst`, or comment blocks in `.py`, `.cpp`).
4.  **Directive Processing**: A `Processor` module takes the discovered files (or extracted content), parses them using the `Parser` module to identify and extract specified RST directives.
5.  **Data Aggregation**: An `Aggregator` module collects all found directives and groups them according to user-specified criteria (e.g., by directive name, by source file, or all into one file).
6.  **Output Generation**: The `Aggregator` then writes the grouped directives into JSON files in the specified output directory.

The library (`src/lib.rs`) exposes the core modules (`parser`, `file_walker`, `aggregator`, `processor`, `extractor`, `timing`) and key data structures, allowing other Rust projects to use this parsing functionality programmatically.

## Key Technical Decisions

-   **Rust as Primary Language**: Chosen for its performance, memory safety, and strong type system, which are beneficial for parsing and file processing tasks.
-   **Modular Design**: The system is broken down into distinct modules (`FileWalker`, `Extractor`, `Parser`, `Processor`, `Aggregator`), each with a specific responsibility. This promotes separation of concerns, testability, and maintainability.
-   **Parallelism with Rayon**: Utilized for potentially speeding up file processing tasks, although the specifics of its usage would need to be confirmed by inspecting the `Processor` or other relevant modules.
-   **Serde for Serialization**: `serde` and `serde_json` are used for robust and efficient serialization of parsed directive data into JSON format.
-   **CLI with Clap**: `clap` provides a declarative way to build a user-friendly command-line interface.
-   **Extensibility for File Types**: The design allows for extracting RST from different file types by handling them in the `Extractor`.
-   **Configurable Output**: Users can control how the output JSON files are structured (grouped by directive, file, or all together).

## Design Patterns in Use

-   **Pipeline/Sequential Processing**: The main workflow follows a pipeline pattern: Find Files -> Extract Content -> Process Directives -> Aggregate Data -> Write Output. Each stage processes the output of the previous one.
-   **Builder Pattern**: The `FileWalker` uses a builder-like pattern for configuration (e.g., `with_extensions()`, `with_max_depth()`).
-   **Strategy Pattern (Implicit)**: The `group_by` argument in the CLI allows selecting different aggregation strategies.
-   **Module/Component-Based Architecture**: The codebase is organized into logical modules, each encapsulating a part of the system's functionality.

## Component Relationships

```mermaid
graph TD
    CLI[CLI Application (main.rs)] --> Args[Argument Parsing (Clap)]
    CLI --> FW[FileWalker]
    CLI --> PROC[Processor]
    CLI --> AGG[Aggregator]

    FW --> FS[File System]
    PROC --> FS
    PROC --> EXT[Extractor]
    PROC --> PAR[Parser]
    EXT --> FS
    PAR -->|Parses| RSTContent[RST Content]

    AGG -->|Aggregates| Directives[Processed Directives]
    AGG -->|Writes| JSONOutput[JSON Output Files]

    Lib[Library (lib.rs)] -.-> FW
    Lib -.-> EXT
    Lib -.-> PAR
    Lib -.-> PROC
    Lib -.-> AGG
    Lib -.-> TIM[Timing]

    subgraph CoreLogic
        direction LR
        FileWalkerModule[file_walker.rs]
        ExtractorModule[extractor.rs]
        ParserModule[parser.rs]
        ProcessorModule[processor.rs]
        AggregatorModule[aggregator.rs]
    end

    FW --> FileWalkerModule
    EXT --> ExtractorModule
    PAR --> ParserModule
    PROC --> ProcessorModule
    AGG --> AggregatorModule

    style CLI fill:#f9f,stroke:#333,stroke-width:2px
    style Lib fill:#ccf,stroke:#333,stroke-width:2px
```

-   **`main.rs` (CLI)**: Orchestrates the overall process. It initializes and calls `FileWalker`, `Processor`, and `Aggregator`.
-   **`lib.rs` (Library)**: Defines the public API by re-exporting components from various modules.
-   **`file_walker.rs`**: Responsible for finding relevant files in the file system based on criteria.
-   **`extractor.rs`**: Responsible for extracting RST content from different file types. It's used by the `Processor`.
-   **`parser.rs`**: Contains the logic to parse RST text and identify directives, arguments, options, and content. It's used by the `Processor`.
-   **`processor.rs`**: Manages the processing of individual files. It uses the `Extractor` to get RST content and the `Parser` to parse that content.
-   **`aggregator.rs`**: Takes the list of found directives (with their source information) from the `Processor` and organizes them into a structured format, then writes them to JSON files.
-   **`timing.rs`**: Likely provides utilities for benchmarking or timing operations (its usage is not directly evident in `main.rs` but it's part of the library).

## Critical Implementation Paths

-   **File Traversal and Filtering**: Efficiently walking directory trees and filtering files by extension and depth (`file_walker.rs`).
-   **RST Content Extraction**: Accurately identifying and extracting RST blocks from various source file types (`extractor.rs`).
-   **Directive Parsing**: Correctly parsing the structure of RST directives, including their name, arguments, options, and content body (`parser.rs`).
-   **Data Aggregation and Serialization**: Efficiently collecting and structuring large numbers of directives and serializing them to JSON (`aggregator.rs`, `serde_json`).
-   **Error Handling**: Robustly handling I/O errors, parsing errors, and invalid user input throughout the pipeline.

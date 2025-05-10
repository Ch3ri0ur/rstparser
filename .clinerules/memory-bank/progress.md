# Progress

## What Works

- File crawling
- Extracting rst from cpp, py and rst files
- parsing the rst for known directives.
- aggregating the data into JSON files
- Basic file watching mode (`--watch` flag) implemented:
    - Initial scan and processing of files on startup, now correctly handling `.rst`, `.py`, and `.cpp` files by default.
    - Monitors for file creation, modification, and deletion events using the `notify` crate for configured extensions.
    - Re-parses affected files and updates an in-memory representation of directives using a unique ID system to prevent duplicates on modification.
    - Re-aggregates and writes updated JSON output files.

## What's Left to Build

- Thorough testing of file watching functionality under various scenarios.
- Potential performance optimizations for watch mode (e.g., event debouncing, more efficient data structures for large projects if needed).
- (Other future features as per projectbrief.md)

## Current Status

- Implemented initial version of file watching and incremental update functionality in `src/main.rs`.
- Resolved build errors related to type mismatches and borrow checker issues in the file watching logic within `src/main.rs`.
- Refactored in-memory directive storage in `src/main.rs` to use a nested `HashMap` structure (`HashMap<PathBuf, HashMap<String, aggregator::DirectiveWithSource>>`) for efficient updates and to prevent directive duplication in watch mode.
- Implemented a unique ID generation for directives (preferring `:id:` option, falling back to file/name/line).
- Updated `src/aggregator.rs` to support the new in-memory data structure.
- The project now builds successfully with these enhancements to the file watching feature.
- Default file extensions for scanning and watching updated in `src/main.rs` to include `rst,py,cpp`, resolving an issue where example files were not processed correctly in watch mode.
- Updated `Cargo.toml` with the `notify` dependency.
- Addressed a bug in watch mode where original directives were duplicated after file changes due to inconsistent path representations; fixed by implementing consistent path canonicalization in `src/main.rs` for in-memory directive storage and event processing.
- Memory bank files (`activeContext.md`, `progress.md`) are being updated to reflect these changes.

## Known Issues

- The file watching functionality has been improved by addressing a key directive duplication bug. However, it is still new and requires comprehensive testing across various scenarios (create, modify, delete files/directories, rapid changes) to ensure full robustness.
- Potential for multiple event triggers for single file operations (may need debouncing in future).
- Performance with very large numbers of files/directives in watch mode has not been benchmarked.
- Previously identified bugs in `parse_rst_multiple` were addressed; their status is now considered resolved unless new issues arise from testing.

## Evolution of Project Decisions

- (To be filled)

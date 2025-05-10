# Active Context

## Current Work Focus

- Refining incremental updates in file watching mode to prevent directive duplication.

## Recent Changes

- Added `notify` crate as a dependency in `Cargo.toml`.
- Modified `src/main.rs` to include a `--watch` CLI flag.
- Implemented initial file system event monitoring using `notify` in `src/main.rs`.
- Restructured in-memory storage (`current_directives_with_source` in `src/main.rs`) from `Vec<DirectiveWithSource>` to `Arc<Mutex<HashMap<PathBuf, HashMap<String, aggregator::DirectiveWithSource>>>>`.
- Implemented a unique ID generation strategy for directives: uses `:id:` option if present, otherwise a composite of file path, directive name, and line number.
- Modified file event handling (Create, Modify, Remove) in `src/main.rs` to use the new map structure, ensuring directives are correctly updated or removed, thus preventing duplicates upon file modification.
- Updated `src/aggregator.rs` with new methods (`aggregate_to_json_from_map`, `aggregate_map_to_json`) to work with the new data structure, and refactored existing aggregation logic into an internal helper.
- Updated default file extensions in `src/main.rs` to `rst,py,cpp` to ensure correct processing of example files in watch mode.
- Resolved a `unused_assignments` warning in `src/main.rs`.
- Previous work involved fixing bugs in `src/parser.rs`.

## Next Steps

- Test the file watching functionality thoroughly with various scenarios (create, modify, delete files/directories, rapid changes).
- Update `.clinerules/memory-bank/progress.md` to reflect the fix for directive duplication and the new data structures.
- Consider potential refinements like event debouncing or more efficient in-memory data structures for `current_directives_with_source` if performance issues arise with very large projects.
- Confirm task completion.

## Active Decisions and Considerations

- Ensuring all memory bank files are consistent with the information available from the project structure and source code.
- Adhering to the structure and purpose of each memory bank file as defined in `.clinerules/cline-memory-bank.md`.
- Avoiding speculation for sections where reviewed files do not provide explicit information (e.g., "What's Left to Build" in `progress.md`).
- Decided to use a `HashMap` keyed by `PathBuf` (for file) and then by a generated `DirectiveInstanceId` (String) to store `aggregator::DirectiveWithSource` objects, enabling efficient updates.
- The `DirectiveInstanceId` prioritizes a user-defined `:id:` field in directive options, falling back to a generated ID (file path + name + line number).

## Important Patterns and Preferences

- The memory bank is crucial for maintaining context between sessions.
- Updates should be thorough and reflect the current state accurately based on available information.

## Learnings and Project Insights

- The project is a reStructuredText (RST) parser implemented in Rust.
- Key aspects include performance (benchmarks exist) and extensibility (custom directives in Python/C++).
- The project includes a library and a CLI tool.
- `main.rs` confirms the CLI argument structure (`clap`) and the orchestration of `FileWalker`, `Extractor`, `Parser`, `Processor`, and `Aggregator` modules.
- `lib.rs` confirms the public API and re-exported modules.
- The core functionality described in `progress.md` ("What Works") is validated by the structure observed in `main.rs`.
- The change to a nested HashMap structure for `current_directives_with_source` is crucial for correct state management in watch mode, especially for updates.

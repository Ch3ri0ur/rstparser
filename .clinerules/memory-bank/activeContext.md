# Active Context

## Current Work Focus

- Implementation of the "directive functions" feature, starting with backlinks.
- This involves creating new modules (`link_data.rs`, `directive_functions.rs`), modifying existing ones (`aggregator.rs`, `processor.rs`, `main.rs`, `lib.rs`), and adding configuration (`rstparser_links.toml`).
- Ensuring the new functionality integrates with both normal and watch modes.
- Addressing compiler errors that arose during implementation.

## Recent Changes

- **Previous Context (before current task):**
    - Added `notify` crate for file watching.
    - Modified `src/main.rs` for `--watch` flag and initial event monitoring.
    - Restructured in-memory storage to `Arc<Mutex<HashMap<PathBuf, HashMap<String, aggregator::DirectiveWithSource>>>>`.
    - Implemented unique ID generation and path canonicalization to prevent directive duplication in watch mode.
    - Updated `src/aggregator.rs` for the new map structure.
- **Current Task (Directive Functions - Backlinks):**
    - Added `toml = "0.8"` dependency to `Cargo.toml`.
    - Created `rstparser_links.toml` for link field configuration.
    - Fixed numerous compiler errors in tests (`src/parser.rs`, `benches/parser_benchmarks.rs`, `examples/timing_example.rs`, `tests/test_cpp_py_extraction.rs`) by updating calls from an old `parse_rst` function to the current `parse_rst_multiple`.
    - Created `src/link_data.rs`:
        - Defined `LinkTypeConfig` and `LinkConfig` for TOML parsing.
        - Defined `LinkNodeData` and `LinkGraph` for storing link relationships.
        - Implemented `load_link_config` function.
    - Created `src/directive_functions.rs`:
        - Defined `DirectiveFunction` trait.
        - Implemented `BacklinkFunction` to process links and update `LinkGraph`.
        - Implemented `FunctionApplicator` to manage and apply directive functions.
    - Modified `src/aggregator.rs`:
        - Added `id: String` field to `DirectiveWithSource`.
        - Introduced `DirectiveOutput` struct for serialization, which includes original options plus dynamically added backlink fields.
        - Added `create_directive_outputs` helper to generate `Vec<DirectiveOutput>` using `LinkGraph`.
        - Added `aggregate_to_json_from_map_with_links` and `aggregate_map_to_json_with_links` methods to use the `LinkGraph` for enriching output.
        - Updated existing aggregation methods to use `DirectiveOutput` internally.
        - Updated tests to reflect `id` field and `DirectiveOutput`.
        - User fixed import paths for `link_data` module (changed from `crate::` to `rstparser::`).
    - Modified `src/processor.rs`:
        - Refactored `process_file` to handle path canonicalization and unique ID generation (using `:id:` option or fallback: canonical_path:name:line), populating `DirectiveWithSource.id` and `DirectiveWithSource.source_file` (as canonical path).
        - Refactored `process_files` (for non-watch mode) to use the updated `process_file`.
        - Added `process_file_watch` (returns `Vec<Arc<Mutex<DirectiveWithSource>>>` with IDs and canonical paths).
        - Added `process_files_watch` (returns `HashMap<PathBuf, Vec<Arc<Mutex<DirectiveWithSource>>>>` for initial scan in watch mode).
        - Updated tests to reflect new ID generation and `DirectiveWithSource` structure.
        - Fixed error handling for `Send + Sync` in parallel processing.
    - Modified `src/main.rs`:
        - Added imports for `link_data` and `directive_functions`.
        - Loads `LinkConfig` from `rstparser_links.toml`.
        - Initializes `FunctionApplicator`.
        - Initializes `LinkGraph`.
        - Calls `function_applicator.apply_to_all()` after initial parsing (for both watch and non-watch modes) to populate the `LinkGraph`.
        - Modified calls to aggregator to use new `_with_links` methods, passing the `LinkGraph`.
        - Adapted data structures and processing flow to align with changes in `Processor` and `Aggregator`.
    - Modified `src/lib.rs` to make `link_data` and `directive_functions` modules public.

## Next Steps

- **Immediate (Post-Reset):**
    - Run `cargo check` and `cargo test` to confirm the project compiles and tests pass after the user's fix for `src/aggregator.rs` imports and the recent refactoring.
    - If errors persist, address them.
- **Backlink Feature Completion:**
    - Refine incremental update logic for `LinkGraph` in `src/main.rs` (watch mode). Currently, it does a full recalculation via `apply_to_all`. This needs to be optimized to:
        - Identify affected directives (sources of changed/deleted links, targets of changed/deleted links).
        - Efficiently remove stale links from `LinkGraph`.
        - Apply functions only to the necessary subset of directives.
    - Thoroughly test backlink generation and updates in various scenarios (create, modify, delete files/directories, changes to link fields, changes to target directive IDs, self-references, broken links).
- **General:**
    - Update `progress.md` to reflect the current state of the backlink feature.
    - Consider performance implications of link processing, especially in watch mode.

## Active Decisions and Considerations

- **Link Configuration**: Using `rstparser_links.toml` with a `[[links]]` array (each entry having a `name` field) to define linkable fields. Backlink fields are generated as `original_field_name_back`.
- **Data Storage for Links**: A separate `LinkGraph` (`HashMap<DirectiveInstanceId, LinkNodeData>`) is used to store relationships, managed by an `Arc<Mutex<>>` in watch mode. `LinkNodeData` stores outgoing and incoming links.
- **Output Enrichment**: Backlink information is added to the `options` map of a temporary `DirectiveOutput` struct during serialization, not directly to `DirectiveWithSource`.
- **ID Generation**: Centralized in `Processor::process_file`, using `:id:` option or fallback (canonical_path:name:line).
- **Path Canonicalization**: Handled in `Processor::process_file` to ensure consistent path representation.
- **Error Handling**: Warnings for self-referential or broken links are printed to `eprintln`. Parallel processing errors in `Processor` are collected and returned.

## Important Patterns and Preferences

- Modular design with clear responsibilities for parsing, processing, link management, and aggregation.
- Incremental updates are a key long-term requirement.
- Configuration via TOML file for link definitions.

## Learnings and Project Insights

- The introduction of link processing significantly increases complexity, especially for state management in watch mode.
- Careful management of borrows and mutability is crucial when dealing with shared data structures like `LinkGraph` and `current_directives_with_source`.
- Path handling (canonicalization) and unique ID generation are fundamental for reliable directive tracking.
- The `Processor` module is now responsible for preparing `DirectiveWithSource` instances with all necessary metadata (ID, canonical path).
- The `Aggregator` is responsible for the final JSON output format, including enrichment with data from the `LinkGraph`.

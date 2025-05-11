# Progress

## What Works

- File crawling.
- Extracting RST from C++, Python, and RST files.
- Parsing RST for known directives.
- Aggregating data into JSON files.
- Basic file watching mode (`--watch` flag):
    - Initial scan and processing.
    - Monitors file events (create, modify, delete).
    - Re-parses affected files and updates in-memory directive storage.
    - Uses unique ID generation (user-defined `:id:` or fallback `canonical_path:name:line`) and path canonicalization to prevent directive duplication.
    - Re-aggregates and writes updated JSON output.
- **Initial implementation of Directive Functions (Backlinks):**
    - `toml` dependency added for configuration.
    - `rstparser_links.toml` created for defining linkable fields.
    - `src/link_data.rs` module:
        - Defines `LinkConfig`, `LinkTypeConfig` for TOML parsing.
        - Defines `LinkGraph`, `LinkNodeData` for storing link relationships.
        - `load_link_config` function implemented.
    - `src/directive_functions.rs` module:
        - `DirectiveFunction` trait and `FunctionApplicator` struct.
        - `BacklinkFunction` implementation to populate `LinkGraph`.
    - `src/aggregator.rs` updated:
        - `DirectiveWithSource` includes an `id: String` field.
        - `DirectiveOutput` struct for enriched JSON serialization.
        - Methods like `aggregate_map_to_json_with_links` use `LinkGraph` to add backlink fields (e.g., `fieldname_back`) to `DirectiveOutput.options`.
    - `src/processor.rs` refactored:
        - `process_file` now handles path canonicalization and populates `DirectiveWithSource.id` and canonical `source_file`.
        - `process_file_watch` and `process_files_watch` added for watch mode, returning data structures with `Arc<Mutex<DirectiveWithSource>>`.
    - `src/main.rs` updated:
        - Loads link configuration.
        - Initializes `FunctionApplicator` and `LinkGraph`.
        - Implemented incremental updates to `LinkGraph` in watch mode:
            - Uses `link_data::remove_links_for_ids` to clear stale link data.
            - Uses `FunctionApplicator::apply_to_subset` to reprocess only affected directives.
            - Retains overall graph consistency by removing nodes for directives that no longer exist.
        - Calls `function_applicator.apply_to_all()` for initial scan and non-watch mode.
        - Uses new aggregator methods that accept the `LinkGraph`.
    - Compiler errors related to old `parse_rst` function in tests and examples fixed.
    - Import paths for `link_data` in `src/aggregator.rs` fixed by the user.
    - `src/link_data.rs` updated with `remove_links_for_ids` function.
    - `src/directive_functions.rs` updated with `apply_to_subset` method in `FunctionApplicator`.

## What's Left to Build

- **Thorough Testing (Backlinks & Incremental Updates):**
    - Comprehensive testing of backlink generation and incremental updates in various scenarios:
        - File/directory creation, modification, deletion.
        - Changes to link fields within directives.
        - Changes to target directive IDs.
        - Self-referential links and handling of broken links.
        - Edge cases and concurrent modifications if possible.
    - Testing of existing file watching functionality under various scenarios.
- **Performance Optimizations:**
    - Benchmarking and optimization of incremental `LinkGraph` updates, especially with large numbers of directives and frequent changes.
    - For watch mode event debouncing (if identified as an issue).
- (Other future features as per projectbrief.md, e.g., inheritance function).

## Current Status

- Core infrastructure for directive functions and backlink processing is in place.
- Backlinks are generated correctly during initial scans (both watch and non-watch modes).
- **Incremental updates for `LinkGraph` in watch mode are implemented.**
    - Watch mode now attempts to update the `LinkGraph` by reprocessing only changed/affected directives rather than a full recalculation.
- Project compiles successfully and all existing tests pass after recent changes.

## Known Issues

- **Performance of Incremental Link Updates in Watch Mode**: While incremental logic is implemented, its performance under heavy load or with very complex link structures has not been thoroughly benchmarked.
- Potential for multiple event triggers for single file operations (may need debouncing in future).

## Evolution of Project Decisions

- Decided to use a separate `LinkGraph` (HashMap) to store link relationships, managed by an `Arc<Mutex<>>` in watch mode.
- Backlink information is added to a temporary `DirectiveOutput` struct during serialization, rather than directly modifying `DirectiveWithSource`'s options field permanently.
- ID generation and path canonicalization are centralized in the `Processor` module.
- Link field types are configured via `rstparser_links.toml`.

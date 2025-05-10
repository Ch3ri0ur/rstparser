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
        - Calls `function_applicator.apply_to_all()` after initial parsing to populate `LinkGraph`.
        - Uses new aggregator methods that accept the `LinkGraph`.
    - Compiler errors related to old `parse_rst` function in tests and examples fixed.
    - Import paths for `link_data` in `src/aggregator.rs` fixed by the user.

## What's Left to Build

- **Backlink Feature - Incremental Updates (Watch Mode):**
    - The current implementation in watch mode re-calculates the entire `LinkGraph` via `apply_to_all` on every change. This needs to be optimized for performance by implementing true incremental updates to the `LinkGraph`. This involves:
        - Identifying only the directly changed directives and those affected by link changes (sources/targets of added/modified/deleted links).
        - Efficiently removing stale links from `LinkGraph`.
        - Applying directive functions only to the necessary subset of directives.
- **Thorough Testing:**
    - Comprehensive testing of backlink generation and updates in various scenarios (create, modify, delete files/directories, changes to link fields, changes to target directive IDs, self-references, broken links), especially in watch mode after incremental updates are implemented.
    - Testing of existing file watching functionality under various scenarios.
- **Performance Optimizations:**
    - For watch mode event debouncing.
    - For `LinkGraph` updates and queries if performance issues arise with very large projects.
- (Other future features as per projectbrief.md, e.g., inheritance function).

## Current Status

- Core infrastructure for directive functions and backlink processing is in place.
- Backlinks are generated correctly during initial scans (both watch and non-watch modes).
- Watch mode re-aggregates with updated backlinks, but link processing is a full recalculation.
- Project compiles successfully after recent changes and fixes.

## Known Issues

- **Performance of Link Updates in Watch Mode**: Full recalculation of `LinkGraph` on each change is not scalable.
- Potential for multiple event triggers for single file operations (may need debouncing in future).
- Performance with very large numbers of files/directives in watch mode (especially with link processing) has not been benchmarked.

## Evolution of Project Decisions

- Decided to use a separate `LinkGraph` (HashMap) to store link relationships, managed by an `Arc<Mutex<>>` in watch mode.
- Backlink information is added to a temporary `DirectiveOutput` struct during serialization, rather than directly modifying `DirectiveWithSource`'s options field permanently.
- ID generation and path canonicalization are centralized in the `Processor` module.
- Link field types are configured via `rstparser_links.toml`.

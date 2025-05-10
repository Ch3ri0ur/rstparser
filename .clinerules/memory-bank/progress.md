# Progress

## What Works

- File crawling
- Extracting rst from cpp, py and rst files
- parsing the rst for known directives.
- aggregating the data into JSON files

## What's Left to Build

- (To be filled)

## Current Status

- Attempted to fix two bugs in `src/parser.rs` within the `parse_rst_multiple` function related to directive name validation/newline handling and advancing the parsing position correctly. Verification pending `cargo test`.

## Known Issues

- Previously identified bugs in `parse_rst_multiple` (related to directive name validation and parsing position advancement) have been addressed; awaiting test confirmation.
- (To be filled with other known issues if any)

## Evolution of Project Decisions

- (To be filled)

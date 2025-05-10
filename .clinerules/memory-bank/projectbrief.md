# Project Brief

## Core Requirements


- Developers need a reliable and performant way to parse specific directives in reStructuredText (RST) documents.
- Other file types with included rst parts (marked with @rst and @endrst) in comment blocks also need to be found and parsed.
- He wants to get the directive type, arguments, fields, options and the content.
- He wants to be parse a large amount of files (nested in folders)
- Performance is important.
- Output for now is a set of JSON files.
- In a future iteration a "file-watching" mode needed to incrementally update the files.

In future versions of this tool the following usecases are also needed:

- Incremental updates
- A set of operations to be applied to the parsed directives like:
    - linking to other directives.
        - configuration about what fields are attributes and what fields are links (for each type)
    - Creating backlinks (and maintaining them on incremental updates).
    - dynamic functions like inheriting data from fields from linked directives. 
- A server with api endpoints to modify and read data about specifc directives or files.
    - Maybe add a filtering mechanism 
- Test against basic rules e.g. no id duplication or link targets are valid ids

Far future:

- A lsp server for (e.g. vscode) to support editing these directive blocks
    - to allow for autocomplete with links
    - et.c

## Project Goals

- To create a robust and efficient RST parser in Rust.
- To enable easy integration with other Rust projects.
- To support common RST features and allow for custom extensions (e.g., `custom_directives.py`, `customcpp.cpp`).

## Scope

- Limited parsing of rst. Mostly just detecting target directives and extracting them.
- Benchmarking and testing for performance and correctness.
- Potentially providing examples and a command-line interface for usage.

# Product Context

## Problem Statement

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


## Solution

- This project provides a Rust library for parsing RST, focusing on correctness, speed, and extensibility.
- It aims to offer a CLI tool for quick parsing and extraction tasks.
- It allows for custom directive handling, making it adaptable to various RST-based documentation or content systems.

## User Experience Goals

- For library users: A simple and intuitive API for parsing RST and accessing its components. Clear documentation and examples.
- For CLI users: Easy-to-use commands for common parsing tasks, with clear output.
- Overall: High performance to handle large documents efficiently. Robust error handling.

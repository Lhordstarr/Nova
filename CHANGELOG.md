# Changelog

## 0.1.2

- `ask` now stores a Number when the input can be parsed as a number, and a String otherwise.
- Added Unknown type handling for input variables.
- Updated documentation and tests.

## 0.1.1

- Added bare-text printing.
- Quotation marks are now optional for ordinary print text.
- Single-quoted strings are supported.
- Updated parser resolution rules for print arguments.
- Updated tests and documentation.

## 0.1.0

Initial release of NOVA.

- Added Rust-based NOVA interpreter.
- Added lexer with indentation-aware tokenization.
- Added recursive-descent parser and AST.
- Added semantic analysis for variables, scopes, and types.
- Added interpreter for printing, variables, arithmetic, conditions, and user input.
- Added CLI commands: `run`, `check`, `version`, `help`.
- Added automated tests for lexer, parser, semantics, interpreter, and CLI.
- Added language, architecture, and project documentation.
- Added `examples/hello.nova`.

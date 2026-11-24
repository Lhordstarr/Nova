# NOVA Architecture

## Pipeline

```text
source text
  -> lexer
  -> tokens
  -> parser
  -> AST
  -> semantic analysis
  -> interpreter
  -> program output
```

`nova check` runs the lexer, parser, and semantic analyzer without executing the program. `nova run` runs the complete pipeline.

## Modules

- `src/lexer`: converts source text into tokens, including indentation tokens.
- `src/parser`: converts tokens into the NOVA abstract syntax tree.
- `src/semantic`: checks variable declarations, scopes, and types.
- `src/interpreter`: executes a valid AST.
- `src/runtime`: values and scoped environments.
- `src/cli`: command-line interface.
- `src/errors`: shared error type and source spans.

## Design Notes

- The frontend (lexer, parser, AST, semantic analysis) is independent of the interpreter. A future compiler backend can reuse it.
- The AST is a plain Rust data structure with no execution behavior.
- Values use one enum (`Value`) so additional types can be added later without changing the overall architecture.
- Scopes are represented as stacks of hash maps in both semantic analysis and runtime execution.
- Input and output are abstracted in `execute_io`, allowing the interpreter to be tested without real standard streams.

## Current Limitations

NOVA 0.1 is an interpreter only. It has no compiler, optimizer, or bytecode representation. Programs execute directly from the AST.

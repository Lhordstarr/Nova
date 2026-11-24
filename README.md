# NOVA Programming Language

NOVA is an independent, English-inspired programming language implemented in Rust. Programs are stored in `.nova` files and executed by the NOVA interpreter.

This repository contains NOVA 0.1, a minimal but real interpreter. It is not a wrapper around Python or JavaScript, and it does not implement AI features.

## Build

```bash
cargo build
```

The executable is created at:

```bash
target/debug/nova
```

For an optimized build:

```bash
cargo build --release
```

## Run

```bash
cargo run -- run examples/hello.nova
```

Or, after building:

```bash
./target/debug/nova run examples/hello.nova
```

## Commands

```bash
nova run program.nova      # Execute a NOVA program
nova check program.nova     # Validate syntax and semantics
nova version               # Display the NOVA version
nova help                  # Display available commands
```

## Example

`examples/hello.nova`:

```nova
print "Hello, World!"

create variable name = "Sharif"

print name

create variable first = 20
create variable second = 5

print first + second
```

Output:

```text
Hello, World!
Sharif
25
```

## Implemented Features

- Printing strings, bare text, variables, and expressions
- Variable declaration with `create variable`
- String and numeric literals
- Arithmetic: `+`, `-`, `*`, `/`
- Standard operator precedence
- Parenthesized expressions
- Unary minus
- Conditions with `if ... :` and indented blocks
- Comparisons: `is greater than`, `is less than`, `is equal to`, `is not equal to`, `is greater than or equal to`, `is less than or equal to`
- User input with `ask "..." and store as name`
- Undefined variable errors
- Division by zero errors
- Type checking for arithmetic and comparisons
- Line comments beginning with `#`

## Testing

```bash
cargo test
```

## Examples

Hello:

```nova
print "Hello, World!"
```

Variables and arithmetic:

```nova
create variable first = 20
create variable second = 5

print first + second
print first - second
print first * second
print first / second
```

Bare text:

```nova
print Welcome to NOVA
print This is my first program
```

Condition:

```nova
create variable age = 20

if age is greater than 18:
    print "You are an adult"
```

User input:

```nova
ask "What is your name?" and store as username

print username
```

## Documentation

- `LANGUAGE_SPEC.md`
- `ARCHITECTURE.md`
- `CHANGELOG.md`

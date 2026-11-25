# NOVA Language Specification

Version: 0.1.3

This document explains the NOVA language in simple terms. NOVA programs are saved in files with the `.nova` extension.

## 1. Running NOVA

```bash
nova run program.nova
nova check program.nova
nova version
nova help
```

- `nova run` runs a program.
- `nova check` checks a program without running it.
- `nova version` shows the version.
- `nova help` shows commands.

## 2. Source Files

- Files use the `.nova` extension.
- Text is UTF-8.
- Lines end with `\n` or `\r\n`.
- Blank lines are ignored.

## 3. Comments

Use `#` to write a comment. Everything after `#` on the same line is ignored.

```nova
print 25 # This is a comment
```

## 4. Indentation and Blocks

NOVA uses indentation for blocks. A block starts after a line ending with `:`.

```nova
if age is greater than 18:
    print "You are an adult"
```

Rules:

- Use spaces for indentation. Tabs are rejected.
- Each block level must use a consistent number of spaces.
- Mixed tabs and spaces are rejected.
- Blank lines do not change indentation.
- The block ends when indentation returns to the previous level.
- A block must contain at least one statement. An empty block is an error.

## 5. Keywords

These words have special meaning in NOVA:

`print`, `create`, `variable`, `if`, `ask`, `and`, `store`, `as`, `is`, `greater`, `than`, `less`, `equal`, `not`, `or`, `to`

All keywords are reserved and cannot be used as variable names.

Simple meanings:

- `print` — show a value or text
- `create` — start a variable declaration
- `variable` — name a variable
- `if` — start a condition
- `ask` — ask the user for input
- `and` — connect parts of `ask`
- `store` — store input
- `as` — name the input variable
- `is` — start a comparison
- `greater` — greater than
- `than` — used with greater/less
- `less` — less than
- `equal` — equal to
- `not` — not equal to
- `or` — used in "or equal to"
- `to` — completes "equal to"

## 6. Operators

```text
+  add
-  subtract or unary minus
*  multiply
/  divide
=  assign in variable creation
:  start a block
(  )  group expressions
```

Unary minus is supported for negative numbers and negated expressions:

```nova
print -5
create variable temp = -3
print -(2 + 3)
```

Example:

```nova
print 20 + 5 * 2
print (20 + 5) * 2
```

`*` and `/` happen before `+` and `-`. Parentheses change the order.

Comparisons also produce Boolean values:

```nova
create variable is_adult = age is greater than 18

print is_adult
```

A comparison can be used anywhere an expression is allowed, including variable declarations and `print`.

## 7. Values and Types

NOVA has three value types:

- Number: `20`, `20.5`
- String: `"Hello"` or `'Hello'`
- Boolean: true/false values created by comparisons

Whole numbers are printed without `.0`.

```nova
print 25
```

Output:

```text
25
```

## 8. Variables

Create a variable like this:

```nova
create variable name = "Sharif"
create variable age = 20
```

Rules:

- A variable name must be a valid identifier.
- Keywords are reserved and cannot be used as variable names.
- You cannot declare the same variable twice in the same scope.
- Reassigning an existing variable without `create variable` is not supported in NOVA 0.1.
- A variable can be used after it is declared.
- Variables declared inside an `if` block are out of scope after the block ends.

Example:

```nova
create variable first = 20
create variable second = 5

print first + second
```

Output:

```text
25
```

## 9. Printing

Print can show quoted strings, numbers, variables, expressions, or bare text.

```nova
print "Hello, World!"
print 25
print 20 + 5
print (20 + 5) * 2
print username
```

Resolution rules:

1. A quoted string is printed as a string.
2. A number, parenthesized expression, arithmetic expression, or comparison expression is evaluated and printed.
3. A single identifier is treated as a variable. If it is in scope, its value is printed. If it is not declared, NOVA reports an undefined variable error.
4. Ordinary unquoted text that is not a valid expression is printed as a string.
5. Text that resembles a malformed expression is reported as an error instead of being printed as text.

Bare text does not need quotes:

```nova
print Welcome to NOVA
print This is my first program
```

Examples:

```nova
create variable username = "Sharif"

print username        # prints Sharif
print Hello, World!   # prints Hello, World!
print 25              # prints 25
```

An undeclared single identifier is not bare text:

```nova
print unknownname     # error: undefined variable 'unknownname'
```

An empty print prints an empty line:

```nova
print
```

## 10. Conditions

Use `if` to run code only when its expression evaluates to `true`.

```nova
create variable age = 20

if age is greater than 18:
    print "You are an adult"
```

The condition must be a Boolean expression. Passing a number, string, or undeclared variable to `if` is an error.

Comparisons:

```nova
is greater than
is less than
is equal to
is not equal to
is greater than or equal to
is less than or equal to
```

Example:

```nova
create variable age = 20

if age is greater than or equal to 18:
    print "At least 18"

if age is less than or equal to 30:
    print "At most 30"
```

NOVA 0.1 does not have `else`.

## 11. User Input

Use `ask` to get input from the user.

```nova
ask "What is your name?" and store as username

print username
```

The prompt is printed without a newline. Surrounding whitespace and newlines are trimmed before parsing. The input is stored as a finite Number if it can be parsed as one; `NaN` and `Infinity` are stored as Strings. Otherwise, the input is stored as a String.

## 12. Scopes

Variables belong to the block where they are created. Variables created outside a block can be used inside it.

```nova
create variable name = "Sharif"

if name is equal to "Sharif":
    print name
```

Variables created inside a block are not available outside it.

## 13. Strings

Strings can use double or single quotes.

```nova
print "Hello"
print 'Hello'
```

Escape sequences:

```text
\"  double quote
\'  single quote
\\  backslash
\n  newline
\t  tab
```

Example:

```nova
print "Line one\nLine two"
```

## 14. Errors

NOVA reports errors for:

- invalid syntax
- undefined variables in expressions
- duplicate variables in the same scope
- reassignment without `create variable`
- reserved keywords used as variable names
- type mismatches
- non-Boolean `if` conditions
- division by zero
- inconsistent indentation
- tabs in indentation
- empty blocks
- malformed expressions in `print`
- unclosed strings and unknown escape sequences

Example error:

```text
error: line 1, column 11: invalid expression in print statement
```

## 15. Complete Examples

### Hello World

```nova
print "Hello, World!"
```

### Variables and Arithmetic

```nova
create variable first = 20
create variable second = 5

print first + second
print first - second
print first * second
print first / second
print (first + second) * 2
```

### Bare Text

```nova
print Welcome to NOVA
print This is my first program
```

### Condition

```nova
create variable age = 20

if age is greater than 18:
    print "You are an adult"
```

### User Input

```nova
ask "What is your name?" and store as username

print username
```

### Calculator

```nova
create variable first = 20
create variable second = 5

print "Addition:"
print first + second

print "Subtraction:"
print first - second

print "Multiplication:"
print first * second

print "Division:"
print first / second
```

### All Keywords Example

```nova
create variable age = 20
create variable username = "Sharif"

ask "What is your name?" and store as entered_name

print username
print entered_name

if age is greater than 18:
    print "Adult"

if age is less than 100:
    print "Less than 100"

if age is equal to 20:
    print "Equal to 20"

if age is not equal to 10:
    print "Not 10"

if age is greater than or equal to 18:
    print "At least 18"

if age is less than or equal to 30:
    print "At most 30"

print All keywords used
```

## 16. Planned Features

These are not implemented yet:

- else and else if
- loops
- functions
- changing an existing variable
- arrays or objects
- boolean literals
- modules or imports
- compiler backend
- self-hosting

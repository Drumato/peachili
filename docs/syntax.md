# Syntax Specification

## EBNF

```
Start Symbol: program
Terminal Symbol: integer_literal

program -> expr

// Expression Rewrite Rule
expr -> addition
addition -> multiplication (addition_op multiplication)*
multiplication -> prefix (multiplication_op prefix)*
prefix -> prefix_op* postfix
postfix -> primary (postfix_op postfix)*
primary -> integer_literal | identifier

// Operators
addition_op -> `+` | `-`
multiplication_op -> `*` | `/`
prefix_op -> `+` | `-` | `&` | `*`
postfix_op -> `.`
```

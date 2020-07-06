# Syntax Specification

## EBNF

```
Start Symbol: program
Terminal Symbol: integer_literal

program -> expr

// Expression Rewrite Rule
expr -> addition
addition -> multiplication (addition_op multiplication)*
multiplication -> primary (multiplication_op primary)*
primary -> integer_literal

// Operators
addition_op -> `+` | `-`
multiplication_op -> `*` | `/`
```

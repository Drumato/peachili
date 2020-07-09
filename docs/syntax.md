# Syntax Specification

## EBNF

```
Start Symbol: program
Terminal Symbol: string_literal/integer_literal/identifier/uint-literal

program -> toplevel*

// Statement Rewrite Rule
statement -> return_st | ifret_st| declare_st | countup_st | asm_st | varinit_st| const_st
return_st -> "return" expression `;`
ifret_st -> "ifret" expression `;`
declare -> "declare" identifier type `;`
countup_st -> "countup" identifier "begin" expression "exclude" expression block `;`
asm_st -> "asm" block `;`
varinit_st -> "varinit" identifier type `=` expression `;`
const_st -> "const" identifier type `=` expression `;`


// Expression Rewrite Rule
expr -> if_expr | addition
if_expf -> "if" paren_expr block ("else" block)?
addition -> multiplication (addition_op multiplication)*
multiplication -> prefix (multiplication_op prefix)*
prefix -> prefix_op* postfix
postfix -> primary (postfix_op postfix)*
primary -> "true" | "false" | integer_literal | string_literal | identifier-path | uint-literal | paren_expr
paren_expr -> `(` expression `)`

// Operators
addition_op -> `+` | `-`
multiplication_op -> `*` | `/`
prefix_op -> `+` | `-` | `&` | `*`
postfix_op -> `.`

// etc
type -> "Int64"
        | "Uint64"
        | "Boolean"
        | "Noreturn"
        | "ConstStr"
        | identifier_path
identifier_path -> identifier (`::` identifier)*
block -> `{` statement* `}`
```

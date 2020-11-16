# The Peachili Syntax Specification

```go

// expression

primary = string_literal | integer_literal | unsigned_integer_literal | identifier_sequence | boolean_literal
integer_literal = [0-9]+
unsigned_integer_literal = 'u' [0-9]+
identifier = [a-zA-Z] ([a-zA-Z0-9] | '_')*
identifier_sequence = identifier ('::' identifier)* ('(' expression (',' expression)* ')')*
boolean_literal = "true" | "false"
string_literal = '"' any-char* '"'
```

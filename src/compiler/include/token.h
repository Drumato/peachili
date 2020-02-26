#include "base.h"

typedef enum {
  TK_INTLIT,
  TK_SYMBOL,
  TK_EOF,
} TokenKind;

typedef struct Token Token;
struct Token {
  TokenKind kind;
  Token *prev;
  Token *next;
  int int_value;
  char *str;
  uint32_t col;
  uint32_t row;
};

Token *new_eof(Token *cur, uint32_t col, uint32_t row);
Token *new_symbol(Token *cur, char *str, int length, uint32_t col,
                  uint32_t row);
Token *new_intlit_token(Token *cur, int int_value, uint32_t col, uint32_t row);
void debug_tokens_to_stderr(bool verbose, Token *top_token);
void dealloc_tokens(Token **token);

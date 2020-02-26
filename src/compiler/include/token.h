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
  uint32_t length;
  char *str;
  uint32_t col;
  uint32_t row;
};

Token *new_token(TokenKind kind, Token *cur, char *str, int length);
void dump_tokens_to_stderr_if_verbose(bool verbose, Token *top_token);
void dealloc_token(Token **token);

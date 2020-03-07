#include "base.h"

typedef enum {
  TK_INTLIT,
  TK_SYMBOL,
  TK_IDENT,
  TK_EOF,

  // keyword
  TK_INT,
  TK_IF,
  TK_ELSE,
  TK_IFRET,
  TK_RETURN,
  TK_FUNC,
  TK_DECLARE,
  TK_COUNTUP,
  TK_FROM,
  TK_TO,
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
Token *new_symbol(Token *cur, char *str, int length, uint32_t col, uint32_t row);
Token *new_keyword(TokenKind kind, Token *cur, uint32_t col, uint32_t row);
Token *new_intlit_token(Token *cur, int int_value, uint32_t col, uint32_t row);
Token *new_ident(Token *cur, char *str, int length, uint32_t col, uint32_t row);
void debug_tokens_to_stderr(bool verbose, Token *top_token);
void dealloc_tokens(Token **token);
void dump_token(Token *t);

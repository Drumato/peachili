#include "token.h"

#include "base.h"
#include "structure.h"

static Token *new_token(TokenKind kind, Token **cur, uint32_t col, uint32_t row);

// デアロケータ

void dealloc_tokens(Token **token) {
  Token *tmp;
  while (*token != NULL) {
    tmp    = *token;
    *token = (*token)->next;
    free(tmp->str);
    free(tmp);
  }
}

// コンストラクタ

Token *new_eof(Token *cur, uint32_t col, uint32_t row) {
  Token *tok = new_token(TK_EOF, &cur, col, row);
  return tok;
}

Token *new_symbol(Token *cur, char *str, int length, uint32_t col, uint32_t row) {
  Token *tok = new_token(TK_SYMBOL, &cur, col, row);

  // 文字列コピー
  tok->str = (char *)calloc(length, sizeof(char));
  strncpy(tok->str, str, length);
  tok->str[length] = 0;

  return tok;
}

Token *new_keyword(TokenKind kind, Token *cur, uint32_t col, uint32_t row) {
  Token *tok = new_token(kind, &cur, col, row);
  return tok;
}

Token *new_ident(Token *cur, char *str, int length, uint32_t col, uint32_t row) {
  Token *tok = new_token(TK_IDENT, &cur, col, row);

  tok->str = (char *)calloc(length, sizeof(char));
  strncpy(tok->str, str, length);
  tok->str[length] = 0;

  return tok;
}
Token *new_intlit_token(Token *cur, int int_value, uint32_t col, uint32_t row) {
  Token *tok     = new_token(TK_INTLIT, &cur, col, row);
  tok->int_value = int_value;
  return tok;
}

// デバッグ関数

void debug_tokens_to_stderr(bool verbose, Token *top_token) {
  if (verbose) {
    fprintf(stderr, "++++++++ debug-tokens ++++++++\n");
    Token *t = top_token;
    do {
      fprintf(stderr, "(%d,%d):\t", t->row, t->col);
      dump_token(t);
      fprintf(stderr, "\n");
      t = t->next;
    } while (t != NULL);
    fprintf(stderr, "\n\n");
  }
};

// static関数

static Token *new_token(TokenKind kind, Token **cur, uint32_t col, uint32_t row) {
  // 1. 新しくトークンを生成
  Token *tok = calloc(1, sizeof(Token));

  // 2. 渡された情報をセット
  tok->kind = kind;
  tok->col  = col;
  tok->row  = row;

  // 3. 現在のトークンの後ろにappend
  (*cur)->next = tok;
  tok->prev    = *cur;
  return tok;
}

void dump_token(Token *t) {
  switch (t->kind) {
    case TK_INTLIT:
      fprintf(stderr, "%d", t->int_value);
      break;
    case TK_IDENT:
      fprintf(stderr, "%s", t->str);
      break;
    case TK_SYMBOL:
      fprintf(stderr, "%s", t->str);
      break;
    case TK_RETURN:
      fprintf(stderr, "RETURN");
      break;
    case TK_IF:
      fprintf(stderr, "IF");
      break;
    case TK_ELSE:
      fprintf(stderr, "ELSE");
      break;
    case TK_IFRET:
      fprintf(stderr, "IFRET");
      break;
    case TK_INT:
      fprintf(stderr, "INT");
      break;
    case TK_FUNC:
      fprintf(stderr, "FUNC");
      break;
    case TK_VAR:
      fprintf(stderr, "VAR");
      break;
    case TK_EOF:
      fprintf(stderr, "EOF");
      break;
    default:
      break;
  }
}

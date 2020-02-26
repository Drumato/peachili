#include "token.h"
#include "base.h"
#include "structure.h"

static void dump_token(Token *t);

void dealloc_token(Token **token) {
  if ((*token)->next != NULL) {
    dealloc_token(&((*token)->next));
    assert(!(*token)->next);

    free(*token);
    *token = NULL;
    assert(!(*token));
  } else {
    free(*token);
    (*token) = NULL;
  }
}
void dump_tokens_to_stderr_if_verbose(bool verbose, Token *top_token) {
  if (verbose) {
    fprintf(stderr, "++++++++ dump-tokens ++++++++\n");
    Token *t = top_token;
    do {
      fprintf(stderr, "(%d,%d):\t", t->row, t->col);
      dump_token(t);
      fprintf(stderr, "\n");
      t = t->next;
    } while (t != NULL);
  }
};

Token *new_token(TokenKind kind, Token *cur, char *str, int length) {
  // 1. 新しくトークンを生成
  Token *tok = calloc(1, sizeof(Token));
  // 2. 渡された情報をセット
  tok->kind = kind;
  tok->str = str;
  tok->length = length;
  // 3. 現在のトークンの後ろにappend
  cur->next = tok;
  tok->prev = cur;
  return tok;
}

static void dump_token(Token *t) {
  switch (t->kind) {
  case TK_INTLIT:
    fprintf(stderr, "%d", t->int_value);
    break;
  case TK_SYMBOL:
    fprintf(stderr, "%c", *t->str);
    break;
  case TK_EOF:
    fprintf(stderr, "EOF");
    break;
  default:
    break;
  }
}

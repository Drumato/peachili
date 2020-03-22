#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "token.h"

static void skip_whitespace(char **ptr);

static Token *tokenize_symbol(char **ptr, Token *cur);

static Token *tokenize_number(char **ptr, Token *cur);

static Token *tokenize_keyword(char **ptr, Token *cur);

static int cut_integer_range(char **ptr, int *value);
static Token *matched_symbol(char **ptr, Token **cur, char *multilen,
                             int length);
static void update_column_and_pointer(char **ptr, uint32_t offset);

static uint32_t fg_col = 1;
static uint32_t fg_row = 1;
static char *fg_keywords[] = {"ifret", "if",     "else",    "int",
                              "func",  "return", "declare", "countup",
                              "from",  "to",     "require", NULL};
static TokenKind fg_tk_kinds[] = {TK_IFRET, TK_IF,     TK_ELSE,    TK_INT,
                                  TK_FUNC,  TK_RETURN, TK_DECLARE, TK_COUNTUP,
                                  TK_FROM,  TK_TO,     TK_REQUIRE};
static char *fg_multilen_symbols[] = {"::", NULL};

Token *tokenize(char *program) {
  Token head;
  head.next = NULL;
  Token *cur = &head;

  while (*program) {
    // skip whitespace
    skip_whitespace(&program);

    Token *tmp = NULL;

    if ((tmp = tokenize_symbol(&program, cur)) != NULL) {
      cur = tmp;
      continue;
    }

    if ((tmp = tokenize_number(&program, cur)) != NULL) {
      cur = tmp;
      continue;
    }

    if ((tmp = tokenize_keyword(&program, cur)) != NULL) {
      cur = tmp;
      continue;
    }

    fprintf(stderr, "can't tokenize -> '%s'\n", program);
    exit(1);
  }

  cur = new_eof(cur, fg_col, fg_row);
  cur->next = NULL;
  return head.next;
}

// 空白類文字の読み飛ばし
static void skip_whitespace(char **ptr) {
  while (isspace(**ptr)) {
    if (**ptr == '\n') {
      fg_col = 1;
      fg_row++;
      (*ptr)++;
      continue;
    }

    update_column_and_pointer(ptr, 1);
  }
}

// 予約語･識別子のトークナイズ
static Token *tokenize_keyword(char **ptr, Token *cur) {
  Token *tok = NULL;

  // 予約語
  for (int i = 0; fg_keywords[i] != NULL; i++) {
    int word_length = strlen(fg_keywords[i]);
    if (!strncmp(*ptr, fg_keywords[i], word_length)) {
      tok = new_keyword(fg_tk_kinds[i], cur, fg_col, fg_row);
      update_column_and_pointer(ptr, word_length);
      return tok;
    }
  }

  // 識別子
  if (isalpha(**ptr)) {
    char *start = *ptr;
    (*ptr)++;
    while (isalpha(**ptr) || isdigit(**ptr) || **ptr == '_') {
      (*ptr)++;
    }
    tok = new_ident(cur, start, *ptr - start, fg_col, fg_row);
    fg_col += start - *ptr;
  }
  return tok;
}

// 記号のトークナイズ
static Token *tokenize_symbol(char **ptr, Token *cur) {
  Token *tok = NULL;

  // 複数文字の記号
  for (int i = 0; fg_multilen_symbols[i] != NULL; i++) {
    char *multilen = fg_multilen_symbols[i];
    if ((tok = matched_symbol(ptr, &cur, multilen, 2)) != NULL) {
      return tok;
    }
  }

  // 1文字の記号
  if (strchr("+-*/;(){}=,\"", **ptr) != NULL) {
    tok = new_symbol(cur, *ptr, 1, fg_col++, fg_row);
    (*ptr)++;
  }
  return tok;
}

// 数値のトークナイズ
static Token *tokenize_number(char **ptr, Token *cur) {
  Token *tok = NULL;
  int value;
  if (isdigit(**ptr)) {
    int length = cut_integer_range(ptr, &value);
    tok = new_intlit_token(cur, value, fg_col, fg_row);

    // 数字の長さ分進める
    fg_col += length;
  }
  return tok;
}

// 切り取った文字列の長さを返す．
static int cut_integer_range(char **ptr, int *value) {
  // 始点を保持
  char *start = *ptr;
  *value = strtol(*ptr, ptr, 10);

  // ポインタ演算によって長さを取得
  int length = *ptr - start;
  return length;
}

static Token *matched_symbol(char **ptr, Token **cur, char *multilen,
                             int length) {
  if (!strncmp(*ptr, multilen, length)) {
    Token *tok = new_symbol(*cur, *ptr, length, fg_col, fg_row);
    fg_col += length;
    *ptr += length;
    return tok;
  }
  return NULL;
}

static void update_column_and_pointer(char **ptr, uint32_t offset) {
  *ptr += offset;
  fg_col += offset;
}
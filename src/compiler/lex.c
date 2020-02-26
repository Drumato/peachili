#include "token.h"
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void skip_whitespace(char **ptr);
static Token *tokenize_symbol(char **ptr, Token *cur);
static Token *tokenize_number(char **ptr, Token *cur);
static void set_token_position(Token **t);

static uint32_t fg_col = 1; 
static uint32_t fg_row = 1; 

Token *lexing(char *program) {
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

    fprintf(stderr, "can't tokenize\n");
    exit(1);
  }

  new_token(TK_EOF, cur, program, 1);
  cur->next = NULL;
  return head.next;
}

// 空白類文字の読み飛ばし
static void skip_whitespace(char **ptr) {
  while (isspace(**ptr)) {
    (*ptr)++;
    fg_col++;
  }
}

// 記号のトークナイズ
static Token *tokenize_symbol(char **ptr, Token *cur) {
  Token *tok = NULL;
  // 1文字の記号
  if (strchr("+-", **ptr) != NULL) {
    tok = new_token(TK_SYMBOL, cur, (*ptr)++, 1);
    set_token_position(&tok);
  }
  return tok;
}

// 数値のトークナイズ
static Token *tokenize_number(char **ptr, Token *cur) {
  Token *tok = NULL;
  if (isdigit(**ptr)) {
    char *start = *ptr;
    int value = strtol(*ptr, ptr, 10);
    int length = *ptr - start;
    tok = new_token(TK_INTLIT, cur, start, length);
    tok->int_value = value;
    set_token_position(&tok);
  }
  return tok;
}

static void set_token_position(Token **t) {
  (*t)->col = fg_col;
  (*t)->row = fg_row;
  fg_col += (*t)->length;
}

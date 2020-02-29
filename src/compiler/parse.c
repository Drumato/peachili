#include "ast.h"
#include "base.h"
#include "token.h"

static Node *expression(void);
static Node *multiplicative(void);
static Node *additive(void);
static Node *primary(void);
static bool eat_if_token_matched(Token **tok, char *pat);
static int expect_intlit_value(Token **tok);

static Token *fg_cur_tok;
static uint32_t fg_col = 1;
static uint32_t fg_row = 1;

Node *parse(Token *top_token) {
  fg_cur_tok = top_token;
  fg_col = fg_cur_tok->col;
  fg_row = fg_cur_tok->row;
  return expression();
}

// expression = unary | expression binary_op expression
static Node *expression(void) { return additive(); }

// additive = multiplicative
//    ( "+" multiplicative | "-" multiplicative )*
static Node *additive(void) {
  Node *node = multiplicative();

  for (;;) {
    if (eat_if_token_matched(&fg_cur_tok, "+"))
      node = new_binary_node(ND_ADD, node, multiplicative(),
                             fg_col, fg_row);
    else if (eat_if_token_matched(&fg_cur_tok, "-"))
      node = new_binary_node(ND_SUB, node, multiplicative(),
                             fg_col, fg_row);
    else
      return node;
  }
}

// multiplicative = primary ( "*" primary | "/" primary )*
static Node *multiplicative(void) {
  Node *node = primary();

  for (;;) {
    if (eat_if_token_matched(&fg_cur_tok, "*"))
      node = new_binary_node(ND_MUL, node, primary(),
                             fg_col, fg_row);
    else if (eat_if_token_matched(&fg_cur_tok, "/"))
      node = new_binary_node(ND_DIV, node, primary(),
                             fg_col, fg_row);
    else
      return node;
  }
}

// primary = intlit
static Node *primary(void) {
  int int_value = expect_intlit_value(&fg_cur_tok);
  return new_intlit_node(int_value, fg_col, fg_row);
}

// もし指定パターンにマッチすれば読みすすめる
static bool eat_if_token_matched(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL ||
      strncmp((*tok)->str, pat, strlen((*tok)->str)))
    return false;
  *tok = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return true;
}

// 数値であれば読み進め,意味値( 整数値 )を返す
static int expect_intlit_value(Token **tok) {
  if ((*tok)->kind != TK_INTLIT)
    fprintf(stderr, "%d:%d: expected integer-literal\n",
            (*tok)->col, (*tok)->row);
  int val = (*tok)->int_value;
  *tok = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return val;
}

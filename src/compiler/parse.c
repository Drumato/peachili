#include "ast.h"
#include "base.h"
#include "token.h"

static Node *expression(void);
static Node *additive(void);
static Node *primary(void);
static bool eat_if_token_matched(Token **tok, char *pat);
static int expect_intlit_value(Token **tok);

static Token *fg_cur_tok;

Node *parse(Token *top_token) {
  fg_cur_tok = top_token;
  return expression();
}

// expression = unary | expression binary_op expression
static Node *expression(void) { return additive(); }

// additive = primary ( "+" primary | "-" primary )*
static Node *additive(void) {
  Node *node = primary();

  for (;;) {
    if (eat_if_token_matched(&fg_cur_tok, "+"))
      node = new_binary_node(ND_ADD, node, primary());
    else if (eat_if_token_matched(&fg_cur_tok, "-"))
      node = new_binary_node(ND_SUB, node, primary());
    else
      return node;
  }
}

// primary = intlit
static Node *primary(void) {
  int int_value = expect_intlit_value(&fg_cur_tok);
  return new_intlit_node(int_value);
}

// もし指定パターンにマッチすれば読みすすめる
static bool eat_if_token_matched(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL ||
      strncmp((*tok)->str, pat, strlen((*tok)->str)))
    return false;
  *tok = (*tok)->next;
  return true;
}

// 数値であれば読み進め,意味値( 整数値 )を返す
static int expect_intlit_value(Token **tok) {
  if ((*tok)->kind != TK_INTLIT)
    fprintf(stderr, "expected integer-literal\n");
  int val = (*tok)->int_value;
  *tok = (*tok)->next;
  return val;
}

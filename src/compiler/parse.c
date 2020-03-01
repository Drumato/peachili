#include "agtype.h"
#include "ast.h"
#include "base.h"

static Node *statement(void);
static Function *function(void);
static Node *return_statement(void);
static Node *expression(void);
static Node *multiplicative(void);
static Node *additive(void);
static Node *unary(void);
static Node *primary(void);
static bool eat_if_symbol_matched(Token **tok, char *pat);
static struct AGType *expect_agtype(Token **tok);
static void expect_keyword(Token **tok, TokenKind kind);
static void expect_symbol(Token **tok, char *pat);
static int expect_intlit_value(Token **tok);
static char *expect_identifier(Token **tok);

static Token *fg_cur_tok;
static uint32_t fg_col = 1;
static uint32_t fg_row = 1;

Function *parse(Token *top_token) {
  fg_cur_tok = top_token;
  fg_col     = fg_cur_tok->col;
  fg_row     = fg_cur_tok->row;
  return function();
}

Function *function(void) {
  expect_keyword(&fg_cur_tok, TK_FUNC);
  char *name = expect_identifier(&fg_cur_tok);

  // とりあえず引数なし
  expect_symbol(&fg_cur_tok, "(");
  expect_symbol(&fg_cur_tok, ")");

  AGType *ret_type = expect_agtype(&fg_cur_tok);

  expect_symbol(&fg_cur_tok, "{");
  Node *stmt = statement();
  expect_symbol(&fg_cur_tok, "}");

  Function *func = new_function(name, stmt, ret_type, fg_col, fg_row);
  return func;
}

// statement = return_stmt
static Node *statement(void) { return return_statement(); }

// return_statement = "return" expression ";"
static Node *return_statement(void) {
  expect_keyword(&fg_cur_tok, TK_RETURN);
  Node *expr = expression();
  expect_symbol(&fg_cur_tok, ";");
  return new_return(expr, fg_col, fg_row);
}

// expression = unary | expression binary_op expression
static Node *expression(void) { return additive(); }

// additive = multiplicative
//    ( "+" multiplicative | "-" multiplicative )*
static Node *additive(void) {
  Node *node = multiplicative();

  for (;;) {
    if (eat_if_symbol_matched(&fg_cur_tok, "+"))
      node = new_binary_node(ND_ADD, node, multiplicative(), fg_col, fg_row);
    else if (eat_if_symbol_matched(&fg_cur_tok, "-"))
      node = new_binary_node(ND_SUB, node, multiplicative(), fg_col, fg_row);
    else
      return node;
  }
}

// multiplicative = unary ( "*" unary | "/" unary )*
static Node *multiplicative(void) {
  Node *node = unary();

  for (;;) {
    if (eat_if_symbol_matched(&fg_cur_tok, "*"))
      node = new_binary_node(ND_MUL, node, unary(), fg_col, fg_row);
    else if (eat_if_symbol_matched(&fg_cur_tok, "/"))
      node = new_binary_node(ND_DIV, node, unary(), fg_col, fg_row);
    else
      return node;
  }
}

// unary = "+" primary | "-" primary
static Node *unary(void) {
  if (eat_if_symbol_matched(&fg_cur_tok, "-")) {
    return new_unary_node(ND_NEG, primary(), fg_col, fg_row);
  }
  eat_if_symbol_matched(&fg_cur_tok, "+");  // +の可能性は読みとばす

  return primary();
}

// primary = intlit
static Node *primary(void) {
  int int_value = expect_intlit_value(&fg_cur_tok);
  return new_intlit_node(int_value, fg_col, fg_row);
}

static struct AGType *expect_agtype(Token **tok) {
  if ((*tok)->kind != TK_INT) {
    fprintf(stderr, "%d:%d: unexpected", (*tok)->row, (*tok)->col);
    dump_token(*tok);
    fprintf(stderr, "\n");
  }
  *tok   = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return new_integer_type();
}

// もし指定パターンにマッチすれば読みすすめる
static bool eat_if_symbol_matched(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL || strncmp((*tok)->str, pat, strlen((*tok)->str))) return false;
  *tok   = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return true;
}

// 数値であれば読み進め,意味値( 整数値 )を返す
static int expect_intlit_value(Token **tok) {
  if ((*tok)->kind != TK_INTLIT)
    fprintf(stderr, "%d:%d: expected integer-literal\n", (*tok)->row, (*tok)->col);
  int val = (*tok)->int_value;
  *tok    = (*tok)->next;
  fg_col  = (*tok)->col;
  fg_row  = (*tok)->row;
  return val;
}

// 指定された予約語であるかチェック，そうでなければエラー
static void expect_keyword(Token **tok, TokenKind kind) {
  if ((*tok)->kind != kind) {
    fprintf(stderr, "%d:%d: unexpected", (*tok)->row, (*tok)->col);
    dump_token(*tok);
    fprintf(stderr, "\n");
  }
  *tok   = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
}

// 指定された記号であるかチェック，そうでなければエラー
static void expect_symbol(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL || strncmp((*tok)->str, pat, strlen((*tok)->str))) {
    fprintf(stderr, "%d:%d: expected %s unexpected ", (*tok)->row, (*tok)->col, pat);
    dump_token(*tok);
    fprintf(stderr, "\n");
  }
  *tok   = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
}

// 識別子であるかチェックし，そうであれば識別子名を返す
static char *expect_identifier(Token **tok) {
  if ((*tok)->kind != TK_IDENT)
    fprintf(stderr, "%d:%d: expected identifier\n", (*tok)->row, (*tok)->col);
  char *name = (*tok)->str;
  *tok       = (*tok)->next;
  fg_col     = (*tok)->col;
  fg_row     = (*tok)->row;
  return name;
}

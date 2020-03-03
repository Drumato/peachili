#include "agtype.h"
#include "ast.h"
#include "base.h"
#include "variable.h"
#include "vector.h"

static Function *function(void);

// statements
static Node *statement(Function **func);
static Node *return_statement(void);
static Node *vardecl_statement(Function **func);
static void compound_statement(Token **tok, Function **func);

// expressions
static Node *expression(void);
static Node *assignment(void);
static Node *multiplicative(void);
static Node *additive(void);
static Node *unary(void);
static Node *primary(void);

// utilities
static inline bool check_curtoken_is(Token **tok, TokenKind kind);
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
  uint32_t def_func_col = fg_col;
  uint32_t def_func_row = fg_row;

  expect_keyword(&fg_cur_tok, TK_FUNC);
  char *name = expect_identifier(&fg_cur_tok);

  // とりあえず引数なし
  expect_symbol(&fg_cur_tok, "(");
  expect_symbol(&fg_cur_tok, ")");

  AGType *ret_type = expect_agtype(&fg_cur_tok);

  Function *func = new_function(name, ret_type, def_func_col, def_func_row);

  compound_statement(&fg_cur_tok, &func);

  return func;
}

static void compound_statement(Token **tok, Function **func) {
  expect_symbol(tok, "{");

  while (true) {
    if (eat_if_symbol_matched(&fg_cur_tok, "}")) break;
    Node *stmt = statement(func);
    put_statement(*func, stmt);
  }
}

// statement = return_stmt
static Node *statement(Function **func) {
  if (check_curtoken_is(&fg_cur_tok, TK_RETURN)) {
    return return_statement();
  } else if (check_curtoken_is(&fg_cur_tok, TK_VAR)) {
    // このノードは何もしないので注意．
    return vardecl_statement(func);
  } else {
    Node *expr = expression();
    expect_symbol(&fg_cur_tok, ";");
    return expr;
  }
}

// return_statement = "return" expression ";"
static Node *return_statement(void) {
  uint32_t return_col = fg_col;
  uint32_t return_row = fg_row;

  expect_keyword(&fg_cur_tok, TK_RETURN);
  Node *expr = expression();
  expect_symbol(&fg_cur_tok, ";");
  return new_return(expr, return_col, return_row);
}

// vardecl = "var" identifier type
static Node *vardecl_statement(Function **func) {
  expect_keyword(&fg_cur_tok, TK_VAR);
  char *name       = expect_identifier(&fg_cur_tok);
  AGType *var_type = expect_agtype(&fg_cur_tok);

  Variable *old_var;
  if ((old_var = find_lvar(*func, name)) == NULL) {
    Variable *new_var = new_local_var(name, var_type);
    put_local_var(*func, new_var);
  }

  expect_symbol(&fg_cur_tok, ";");
  return new_nop();
}

// expression = assignment
static Node *expression(void) { return assignment(); }

// assignment = additive "=" expression
static Node *assignment(void) {
  Node *lvar = additive();

  uint32_t col = fg_col;
  uint32_t row = fg_row;

  if (!eat_if_symbol_matched(&fg_cur_tok, "=")) return lvar;

  Node *expr = expression();
  return new_assign(lvar, expr, col, row);
}

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
  uint32_t col = fg_col;
  uint32_t row = fg_row;
  if (check_curtoken_is(&fg_cur_tok, TK_INTLIT)) {
    int int_value = expect_intlit_value(&fg_cur_tok);
    return new_intlit_node(int_value, col, row);
  } else {
    char *name = expect_identifier(&fg_cur_tok);
    return new_ident_node(name, col, row);
  }
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

static inline bool check_curtoken_is(Token **tok, TokenKind kind) { return (*tok)->kind == kind; }

// 指定された予約語であるかチェック，そうでなければエラー
static void expect_keyword(Token **tok, TokenKind kind) {
  if (!check_curtoken_is(tok, kind)) {
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

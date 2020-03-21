#include "agtype.h"
#include "ast.h"
#include "base.h"
#include "bundler.h"
#include "module.h"
#include "util.h"
#include "variable.h"
#include "vector.h"

static Function *function(void);

// statements
static Node *statement(void);

static Node *return_statement(void);

static Node *countup_statement(void);

static Node *ifret_statement(void);

static Node *vardecl_statement(void);

static void compound_statement(Token **tok);

// expressions
static Node *if_expression(void);

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

static IdentName *expect_identifier(Token **tok);

static void set_current_position(Token **tok, uint32_t *col, uint32_t *row);

// file global definitions
static Token *fg_cur_tok;
static uint32_t fg_col = 1;
static uint32_t fg_row = 1;
static bool in_if_scope = false;
static Function **this_func;
static int fg_source_i;

Vector *parse(Token *top_token, int source_i) {
  fg_source_i = source_i;
  fg_cur_tok = top_token;
  set_current_position(&fg_cur_tok, &fg_col, &fg_row);

  Vector *funcs = new_vec();

  // requireはBundlerで処理するので無視
  if (check_curtoken_is(&fg_cur_tok, TK_REQUIRE)) {
    while (!eat_if_symbol_matched(&fg_cur_tok, ";")) {
      fg_cur_tok = fg_cur_tok->next;
    }
  }

  // グローバル変数はないものとする．
  // 今はグローバルには関数列があるのみ．
  while (check_curtoken_is(&fg_cur_tok, TK_FUNC)) {
    vec_push(funcs, (void *)function());
  }
  return funcs;
}

Function *function(void) {
  uint32_t def_func_col, def_func_row;
  set_current_position(&fg_cur_tok, &def_func_col, &def_func_row);

  expect_keyword(&fg_cur_tok, TK_FUNC);
  char *name = expect_identifier(&fg_cur_tok)->name;

  Function *func = new_function(name, NULL, def_func_col, def_func_row);
  func->kind = FN_DEFINED;
  this_func = &func;

  expect_symbol(&fg_cur_tok, "(");

  Vector *args = new_vec();
  while (!eat_if_symbol_matched(&fg_cur_tok, ")")) {
    uint32_t def_arg_col, def_arg_row;
    set_current_position(&fg_cur_tok, &def_arg_col, &def_arg_row);

    char *name = expect_identifier(&fg_cur_tok)->name;

    char *allocated_name = str_alloc_and_copy(name, strlen(name));

    vec_push(args, (void *)allocated_name);
    AGType *var_type = expect_agtype(&fg_cur_tok);

    if (find_lvar(*this_func, name) != NULL) {
      fprintf(stderr, "%d:%d: %s already defined\n", def_arg_row, def_arg_col,
              name);
    }

    Variable *new_var = new_local_var(name, var_type);
    put_local_var(*this_func, new_var);

    eat_if_symbol_matched(&fg_cur_tok, ",");
  }

  (*this_func)->return_type = expect_agtype(&fg_cur_tok);
  (*this_func)->args = args;

  compound_statement(&fg_cur_tok);

  return func;
}

static void compound_statement(Token **tok) {
  expect_symbol(tok, "{");

  while (true) {
    if (eat_if_symbol_matched(&fg_cur_tok, "}"))
      break;
    Node *stmt = statement();
    put_statement(*this_func, stmt);
  }
}

// statement = return_stmt
static Node *statement(void) {
  if (check_curtoken_is(&fg_cur_tok, TK_RETURN)) {
    return return_statement();
  } else if (check_curtoken_is(&fg_cur_tok, TK_COUNTUP)) {
    return countup_statement();
  } else if (check_curtoken_is(&fg_cur_tok, TK_IFRET)) {
    return ifret_statement();
  } else if (check_curtoken_is(&fg_cur_tok, TK_DECLARE)) {
    // このノードは何もしないので注意．
    return vardecl_statement();
  } else {
    Node *expr = expression();
    expect_symbol(&fg_cur_tok, ";");
    return expr;
  }
}

// return_statement = "return" expression ";"
static Node *return_statement(void) {
  uint32_t return_col, return_row;
  set_current_position(&fg_cur_tok, &return_col, &return_row);

  expect_keyword(&fg_cur_tok, TK_RETURN);
  Node *expr = expression();
  expect_symbol(&fg_cur_tok, ";");
  return new_return(expr, return_col, return_row);
}

// countup_statement = "countup" typename primary "from" expr "to" expr
static Node *countup_statement(void) {
  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  expect_keyword(&fg_cur_tok, TK_COUNTUP);
  Node *lvar = primary();
  AGType *var_type = expect_agtype(&fg_cur_tok);

  Variable *old_var;
  // 本当はスコープを新しく定義すべき
  if ((old_var = find_lvar(*this_func, lvar->id_name->name)) == NULL) {
    Variable *new_var = new_local_var(lvar->id_name->name, var_type);
    put_local_var(*this_func, new_var);
  }

  expect_keyword(&fg_cur_tok, TK_FROM);
  Node *start = expression();

  expect_keyword(&fg_cur_tok, TK_TO);
  Node *end = expression();

  Vector *stmts = new_vec();
  expect_symbol(&fg_cur_tok, "{");

  // countupの本体
  while (true) {
    if (eat_if_symbol_matched(&fg_cur_tok, "}"))
      break;
    Node *stmt = statement();
    vec_push(stmts, (void *)stmt);
  }

  expect_symbol(&fg_cur_tok, ";");
  return new_countup(lvar, start, end, stmts, start_col, start_row);
}

// ifret_statement = "ifret" expression ";"
static Node *ifret_statement(void) {
  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  if (!in_if_scope) {
    fprintf(stderr,
            "%d:%d: ifret-statement can only exist in if-expression block\n",
            start_row, start_col);
    exit(1);
  }

  expect_keyword(&fg_cur_tok, TK_IFRET);
  Node *expr = expression();
  expect_symbol(&fg_cur_tok, ";");
  return new_ifret(expr, start_col, start_row);
}

// vardecl = "declare" identifier typename
static Node *vardecl_statement() {
  expect_keyword(&fg_cur_tok, TK_DECLARE);
  char *name = expect_identifier(&fg_cur_tok)->name;
  AGType *var_type = expect_agtype(&fg_cur_tok);

  Variable *old_var;
  if ((old_var = find_lvar(*this_func, name)) == NULL) {
    Variable *new_var = new_local_var(name, var_type);
    put_local_var(*this_func, new_var);
  }

  expect_symbol(&fg_cur_tok, ";");
  return new_nop();
}

// expression = if-expression | assignment
static Node *expression(void) {
  if (check_curtoken_is(&fg_cur_tok, TK_IF)) {
    return if_expression();
  }

  return assignment();
}

// if-expression =
static Node *if_expression(void) {
  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  expect_keyword(&fg_cur_tok, TK_IF);

  expect_symbol(&fg_cur_tok, "(");
  Node *cond = expression();
  expect_symbol(&fg_cur_tok, ")");

  in_if_scope = true;

  Vector *stmts = new_vec();
  Vector *alter = NULL;
  expect_symbol(&fg_cur_tok, "{");

  // ifの本体
  while (true) {
    if (eat_if_symbol_matched(&fg_cur_tok, "}"))
      break;
    Node *stmt = statement();
    vec_push(stmts, (void *)stmt);
  }

  // elseでなければ終了
  if (!check_curtoken_is(&fg_cur_tok, TK_ELSE)) {
    in_if_scope = false;
    return new_if(cond, stmts, alter, start_col, start_row);
  }

  alter = new_vec();

  expect_keyword(&fg_cur_tok, TK_ELSE);
  expect_symbol(&fg_cur_tok, "{");

  // elseの本体
  while (true) {
    if (eat_if_symbol_matched(&fg_cur_tok, "}"))
      break;
    Node *stmt = statement();
    vec_push(alter, (void *)stmt);
  }
  in_if_scope = false;

  return new_if(cond, stmts, alter, start_col, start_row);
}

// assignment = additive "=" expression
static Node *assignment(void) {
  Node *lvar = additive();

  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  if (!eat_if_symbol_matched(&fg_cur_tok, "="))
    return lvar;

  Node *expr = expression();
  return new_assign(lvar, expr, start_col, start_row);
}

// additive = multiplicative
//    ( "+" multiplicative | "-" multiplicative )*
static Node *additive(void) {
  Node *node = multiplicative();

  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  for (;;) {
    if (eat_if_symbol_matched(&fg_cur_tok, "+"))
      node =
          new_binary_node(ND_ADD, node, multiplicative(), start_col, start_row);
    else if (eat_if_symbol_matched(&fg_cur_tok, "-"))
      node =
          new_binary_node(ND_SUB, node, multiplicative(), start_col, start_row);
    else
      return node;
  }
}

// multiplicative = unary ( "*" unary | "/" unary )*
static Node *multiplicative(void) {
  Node *node = unary();

  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  for (;;) {
    if (eat_if_symbol_matched(&fg_cur_tok, "*"))
      node = new_binary_node(ND_MUL, node, unary(), start_col, start_row);
    else if (eat_if_symbol_matched(&fg_cur_tok, "/"))
      node = new_binary_node(ND_DIV, node, unary(), start_col, start_row);
    else
      return node;
  }
}

// unary = "+" primary | "-" primary
static Node *unary(void) {
  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  if (eat_if_symbol_matched(&fg_cur_tok, "-")) {
    return new_unary_node(ND_NEG, primary(), start_col, start_row);
  }
  eat_if_symbol_matched(&fg_cur_tok, "+"); // +の可能性は読みとばす

  return primary();
}

// primary = intlit
static Node *primary(void) {
  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  if (check_curtoken_is(&fg_cur_tok, TK_INTLIT)) {
    int int_value = expect_intlit_value(&fg_cur_tok);
    return new_intlit_node(int_value, start_col, start_row);
  } else {
    IdentName *id_name = expect_identifier(&fg_cur_tok);

    if (!eat_if_symbol_matched(&fg_cur_tok, "(")) {
      return new_ident_node(id_name, start_col, start_row);
    }
    // call-expression

    Vector *args = new_vec();
    while (!eat_if_symbol_matched(&fg_cur_tok, ")")) {
      vec_push(args, (void *)expression());

      eat_if_symbol_matched(&fg_cur_tok, ",");
    }

    return new_call(id_name, args, start_col, start_row);
  }
}

static struct AGType *expect_agtype(Token **tok) {
  if ((*tok)->kind != TK_INT) {
    fprintf(stderr, "%d:%d: unexpected", (*tok)->row, (*tok)->col);
    dump_token(*tok);
    fprintf(stderr, "\n");
  }
  *tok = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return new_integer_type();
}

// もし指定パターンにマッチすれば読みすすめる
static bool eat_if_symbol_matched(Token **tok, char *pat) {
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
    fprintf(stderr, "%d:%d: expected integer-literal\n", (*tok)->row,
            (*tok)->col);
  int val = (*tok)->int_value;
  *tok = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return val;
}

static inline bool check_curtoken_is(Token **tok, TokenKind kind) {
  return (*tok)->kind == kind;
}

// 指定された予約語であるかチェック，そうでなければエラー
static void expect_keyword(Token **tok, TokenKind kind) {
  if (!check_curtoken_is(tok, kind)) {
    fprintf(stderr, "%d:%d: unexpected", (*tok)->row, (*tok)->col);
    dump_token(*tok);
    fprintf(stderr, "\n");
  }
  *tok = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
}

// 指定された記号であるかチェック，そうでなければエラー
static void expect_symbol(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL ||
      strncmp((*tok)->str, pat, strlen((*tok)->str))) {
    fprintf(stderr, "%d:%d: expected %s unexpected ", (*tok)->row, (*tok)->col,
            pat);
    dump_token(*tok);
    fprintf(stderr, "\n");
  }
  *tok = (*tok)->next;
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
}

// 識別子であるかチェックし，そうであれば識別子名を返す
static IdentName *expect_identifier(Token **tok) {
  if ((*tok)->kind != TK_IDENT)
    fprintf(stderr, "%d:%d: expected identifier\n", (*tok)->row, (*tok)->col);
  char *name = (*tok)->str;
  *tok = (*tok)->next;

  IdentName *base = new_ident_name(name, NULL);
  IdentName *prev = base;
  IdentName *next = NULL;
  while (eat_if_symbol_matched(&fg_cur_tok, "::")) {
    if ((*tok)->kind != TK_IDENT)
      fprintf(stderr, "%d:%d: module name must be an identifier\n", (*tok)->row,
              (*tok)->col);

    char *next_name = str_alloc_and_copy((*tok)->str, strlen((*tok)->str));

    // 使ったAPIを記録
    Module *current_mod = (Module *)vec_get(sources_g, fg_source_i);
    Module *next_mod;
    if ((next_mod = find_required_mod(current_mod, name)) != NULL) {
      vec_push(next_mod->used, (void *)next_name);
    }

    next = append_ident_name(next_name, &prev);
    prev = next;

    *tok = (*tok)->next;
  }

  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return base;
}

static void set_current_position(Token **tok, uint32_t *col, uint32_t *row) {
  *col = (*tok)->col;
  *row = (*tok)->row;
}

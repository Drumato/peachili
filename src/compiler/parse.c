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

static Node *asm_statement(void);
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
static char *expect_strlit_contents(Token **tok);
static Node *construct_intlit_node(uint32_t col, uint32_t row);
static Node *construct_strlit_node(uint32_t col, uint32_t row);
static Node *construct_ident_node(uint32_t col, uint32_t row);
static IdentName *expect_identifier(Token **tok);
static void parse_body(Vector **sequence);
static void set_current_position(Token **tok, uint32_t *col, uint32_t *row);

// file global definitions
static Token *fg_cur_tok; // ファイルグローバルなトークン
static Module *fg_current_mod;
static uint32_t fg_col = 1;
static uint32_t fg_row = 1;
static uint32_t fg_str_n = 1;
static bool in_if_scope =
    false; // ifret を検出するために使用． 意味解析でやるべきかも
static Function **this_func;

Vector *parse(Token *top_token, Module **mod) {
  fg_current_mod = *mod;
  fg_cur_tok = top_token;
  set_current_position(&fg_cur_tok, &fg_col, &fg_row);

  Vector *funcs = new_vec();

  // requireはBundlerで処理するので無視
  if (check_curtoken_is(&fg_cur_tok, TK_REQUIRE)) {
    while (!eat_if_symbol_matched(&fg_cur_tok, ")")) {
      progress_token(&fg_cur_tok);
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

  // func <identifier> ( args ) ag_type までのパース
  Function *func;
  Vector *args = new_vec();
  {
    expect_keyword(&fg_cur_tok, TK_FUNC);
    char *name = expect_identifier(&fg_cur_tok)->name;

    func = new_function(name, NULL, def_func_col, def_func_row);
    func->kind = FN_DEFINED;
    this_func = &func;

    expect_symbol(&fg_cur_tok, "(");

    while (!eat_if_symbol_matched(&fg_cur_tok, ")")) {
      uint32_t def_arg_col, def_arg_row;
      set_current_position(&fg_cur_tok, &def_arg_col, &def_arg_row);

      char *name = expect_identifier(&fg_cur_tok)->name;

      char *allocated_name = str_alloc_and_copy(name, strlen(name));

      vec_push(args, (void *)allocated_name);
      AGType *var_type = expect_agtype(&fg_cur_tok);

      if (find_lvar(func, name) != NULL) {
        fprintf(stderr, "%d:%d: %s already defined\n", def_arg_row, def_arg_col,
                name);
      }

      Variable *new_var = new_local_var(name, var_type);
      put_local_var(func, new_var);

      eat_if_symbol_matched(&fg_cur_tok, ",");
    }
  }

  func->return_type = expect_agtype(&fg_cur_tok);
  func->args = args;

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
  TokenKind cur_kind = fg_cur_tok->kind;

  // 開始記号から処理を分ける
  switch (cur_kind) {
  case TK_RETURN:
    return return_statement();
  case TK_COUNTUP:
    return countup_statement();
  case TK_IFRET:
    return ifret_statement();
  case TK_DECLARE:
    return vardecl_statement();
  case TK_ASM:
    return asm_statement();
  default: {
    Node *expr = expression();
    expect_symbol(&fg_cur_tok, ";");
    return expr;
  }
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

  Node *countup_node;
  Vector *stmts = new_vec();

  // "countup" typename primary "from" expr "to" expr のパース
  {
    expect_keyword(&fg_cur_tok, TK_COUNTUP);
    Node *loop_var = primary();
    AGType *var_type = expect_agtype(&fg_cur_tok);

    Variable *old_var;
    // 本当はスコープを新しく定義すべき
    if ((old_var = find_lvar(*this_func, loop_var->id_name->name)) == NULL) {
      Variable *new_var = new_local_var(loop_var->id_name->name, var_type);
      put_local_var(*this_func, new_var);
    }

    expect_keyword(&fg_cur_tok, TK_FROM);
    Node *start_expr = expression();

    expect_keyword(&fg_cur_tok, TK_TO);
    Node *end_expr = expression();

    countup_node = new_countup(loop_var, start_expr, end_expr, stmts, start_col,
                               start_row);
  }

  expect_symbol(&fg_cur_tok, "{");

  // countupの本体
  parse_body(&countup_node->body);

  expect_symbol(&fg_cur_tok, ";");
  return countup_node;
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

static Node *asm_statement(void) {
  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);
  expect_keyword(&fg_cur_tok, TK_ASM);
  expect_symbol(&fg_cur_tok, "{");

  Vector *args = new_vec();
  Node *asm_inline_node = new_asm_node(args, start_col, start_row);

  // 複数のアセンブリをパース
  {
    while (true) {
      if (eat_if_symbol_matched(&fg_cur_tok, "}"))
        break;
      Node *str = construct_strlit_node(fg_col, fg_row);
      if (str->kind != ND_STRLIT) {
        fprintf(stderr,
                "%d:%d: each sentence must be a string-literal in asm{}",
                start_row, start_col);
        exit(1);
      }
      eat_if_symbol_matched(&fg_cur_tok, ",");
      vec_push(asm_inline_node->args, (void *)str);
    }
  }

  expect_symbol(&fg_cur_tok, ";");
  return asm_inline_node;
}

// expression = if-expression | assignment
static Node *expression(void) {
  if (check_curtoken_is(&fg_cur_tok, TK_IF)) {
    return if_expression();
  }

  return assignment();
}

static Node *if_expression(void) {
  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  Node *if_node;
  Vector *stmts = new_vec();
  Vector *alter = NULL;

  // "if" ( condition ) までのパース
  {
    expect_keyword(&fg_cur_tok, TK_IF);

    expect_symbol(&fg_cur_tok, "(");
    Node *cond = expression();
    expect_symbol(&fg_cur_tok, ")");

    if_node = new_if(cond, stmts, alter, start_col, start_row);
  }

  in_if_scope = true;
  expect_symbol(&fg_cur_tok, "{");

  // ifの本体のパース
  {
    parse_body(&if_node->body);

    // elseでなければ終了
    if (!check_curtoken_is(&fg_cur_tok, TK_ELSE)) {
      in_if_scope = false;
      return if_node;
    }
  }

  // elseがある場合のパース
  {
    if_node->alter = new_vec();
    expect_keyword(&fg_cur_tok, TK_ELSE);
    expect_symbol(&fg_cur_tok, "{");

    // elseの本体
    parse_body(&if_node->alter);
    in_if_scope = false;
  }

  return if_node;
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

// primary = intlit | strlit
static Node *primary(void) {
  uint32_t start_col, start_row;
  set_current_position(&fg_cur_tok, &start_col, &start_row);

  TokenKind cur_kind = fg_cur_tok->kind;
  Node *n;
  switch (cur_kind) {
  case TK_INTLIT: {
    n = construct_intlit_node(start_col, start_row);
    break;
  }
  case TK_STRLIT: {
    n = construct_strlit_node(start_col, start_row);
    vec_push(fg_current_mod->strings, (void *)n);
    break;
  }
  default: {
    n = construct_ident_node(start_col, start_row);
    break;
  }
  }
  return n;
}

static void parse_body(Vector **sequence) {
  while (true) {
    if (eat_if_symbol_matched(&fg_cur_tok, "}"))
      break;
    Node *stmt = statement();
    vec_push(*sequence, (void *)stmt);
  }
}

static struct AGType *expect_agtype(Token **tok) {
  TokenKind type_kind = (*tok)->kind;
  progress_token(tok);
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;

  AGType *agtype = NULL;
  switch (type_kind) {
  case TK_INT: {
    agtype = new_integer_type();
    break;
  }
  case TK_NORETURN: {
    agtype = new_noreturn_type();
    break;
  }
  default:
    fprintf(stderr, "%d:%d: unexpected ", (*tok)->row, (*tok)->col);
    dump_token(*tok);
    fprintf(stderr, "\n");
    break;
  }

  return agtype;
}

// もし指定パターンにマッチすれば読みすすめる
static bool eat_if_symbol_matched(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL ||
      strncmp((*tok)->str, pat, strlen((*tok)->str)))
    return false;
  progress_token(tok);
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return true;
}

static char *expect_strlit_contents(Token **tok) {
  if ((*tok)->kind != TK_STRLIT) {
    fprintf(stderr, "%d:%d: expected string-literal\n", (*tok)->row,
            (*tok)->col);
  }
  char *contents = (*tok)->str;
  progress_token(tok);
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return contents;
}

// 数値であれば読み進め,意味値( 整数値 )を返す
static int expect_intlit_value(Token **tok) {
  if ((*tok)->kind != TK_INTLIT)
    fprintf(stderr, "%d:%d: expected integer-literal\n", (*tok)->row,
            (*tok)->col);
  int val = (*tok)->int_value;
  progress_token(tok);
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
  progress_token(tok);
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
  progress_token(tok);
  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
}

// 識別子であるかチェックし，そうであれば識別子名を返す
static IdentName *expect_identifier(Token **tok) {
  if ((*tok)->kind != TK_IDENT)
    fprintf(stderr, "%d:%d: expected identifier\n", (*tok)->row, (*tok)->col);
  char *name = (*tok)->str;
  progress_token(tok);

  IdentName *base = new_ident_name(name, NULL);
  IdentName *prev = base;
  IdentName *next = NULL;

  while (eat_if_symbol_matched(&fg_cur_tok, "::")) {
    if ((*tok)->kind != TK_IDENT)
      fprintf(stderr, "%d:%d: module name must be an identifier\n", (*tok)->row,
              (*tok)->col);

    char *next_name = str_alloc_and_copy((*tok)->str, strlen((*tok)->str));

    next = append_ident_name(next_name, &prev);
    prev = next;

    progress_token(tok);
  }

  fg_col = (*tok)->col;
  fg_row = (*tok)->row;
  return base;
}

static void set_current_position(Token **tok, uint32_t *col, uint32_t *row) {
  *col = (*tok)->col;
  *row = (*tok)->row;
}

static Node *construct_intlit_node(uint32_t col, uint32_t row) {
  int int_value = expect_intlit_value(&fg_cur_tok);
  return new_intlit_node(int_value, col, row);
}

static Node *construct_strlit_node(uint32_t col, uint32_t row) {
  char *str = expect_strlit_contents(&fg_cur_tok);
  char *contents = str_alloc_and_copy(str, strlen(str));
  return new_strlit_node(contents, fg_str_n++, col, row);
}

static Node *construct_ident_node(uint32_t col, uint32_t row) {
  IdentName *id_name = expect_identifier(&fg_cur_tok);

  if (!eat_if_symbol_matched(&fg_cur_tok, "(")) {
    return new_ident_node(id_name, col, row);
  }
  // call-expression

  Vector *args = new_vec();
  while (!eat_if_symbol_matched(&fg_cur_tok, ")")) {
    vec_push(args, (void *)expression());

    eat_if_symbol_matched(&fg_cur_tok, ",");
  }
  return new_call(id_name, args, col, row);
}

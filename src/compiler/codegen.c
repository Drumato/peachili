#include "ast.h"
#include "module.h"
#include "variable.h"
#include "vector.h"

// function
static void gen_func(void);

static void gen_function_prologue(char *name, uint32_t offset);

static void gen_function_epilogue(void);

// statement
static void gen_stmt(Node *n);

static void gen_stmts_in_vec(Vector *v);

static void gen_countup_stmt(Node *n);

static void gen_return_stmt(Node *n);

static void gen_asm_stmt(Node *n);

// expression
static void gen_lval(Node *n);

static void gen_exprs_in_vec(Vector *v);

static void gen_expr(Node *n);

static void gen_if_expr(Node *n);

static void gen_if_else_expr(Node *n);

static void gen_base_op_expr(NodeKind kind);

static void gen_binary_expr(Node *n);

static void gen_unary_expr(Node *n);

// utilities
static char *find_library_api(IdentName *id_name);

static void store_reg_using_reg(char *addr_reg, char *val_reg);

static void store_reg(uint32_t offset, char *reg);

// file global definitions
static Function *this_func;
static int label = 0;
static char *caller_regs64[] = {"rdi", "rsi", "rdx", "rcx", "r8", "r9", NULL};

#define GEN_COMMENT_AND_NODE_WITH_FUNC(stmt_name, n, f)                        \
  do {                                                                         \
    printf("\n  # start %s\n", #stmt_name);                                    \
    f(n);                                                                      \
    printf("  # end %s\n\n", #stmt_name);                                      \
  } while (0)

void gen_x64_primary(Vector *functions) {
  for (int i = 0; i < functions->length; i++) {
    Function *iter_func = (Function *)vec_get(functions, i);
    this_func = iter_func;

    gen_func();
  }
}

void gen_x64_external(Module *mod) {
  for (int i = 0; i < mod->functions->length; i++) {
    Function *iter_func = (Function *)vec_get(mod->functions, i);
    this_func = iter_func;

    if (function_is_used(mod, this_func->name)) {
      gen_func();
    }
  }
}
void gen_x64_strlit(Module *mod) {
  for (int i = 0; i < mod->strings->length; i++) {
    Node *strlit = (Node *)vec_get(mod->strings, i);
    printf(".LS%d:\n", strlit->str_n);
    printf("  .string \"%s\"\n", strlit->contents);
    printf(".LS%dLEN:\n", strlit->str_n);
    printf("  .long %lu\n", strlen(strlit->contents));
  }
}

static void gen_func(void) {
  gen_function_prologue(this_func->name, this_func->stack_offset);

  for (int i = 0; i < this_func->args->length; i++) {
    char *arg_name = (char *)vec_get(this_func->args, i);
    char *reg = caller_regs64[i];

    Variable *arg = find_lvar(this_func, arg_name);

    store_reg(arg->offset, reg);
  }

  gen_stmts_in_vec(this_func->stmts);
}

static void gen_stmt(Node *n) {
  switch (n->kind) {
  case ND_RETURN:
    GEN_COMMENT_AND_NODE_WITH_FUNC(return_statement, n, gen_return_stmt);
    break;
  case ND_IFRET:
    GEN_COMMENT_AND_NODE_WITH_FUNC(ifret_statement, n->expr, gen_expr);
    break;
  case ND_COUNTUP:
    GEN_COMMENT_AND_NODE_WITH_FUNC(countup_statement, n, gen_countup_stmt);
    break;
  case ND_ASM:
    GEN_COMMENT_AND_NODE_WITH_FUNC(asm_statement, n, gen_asm_stmt);
    break;
  case ND_NOP:
    break;
  default:
    // expression-statementとする
    GEN_COMMENT_AND_NODE_WITH_FUNC(expression_statement, n, gen_expr);
    break;
  }
}

static void gen_return_stmt(Node *n) {
  gen_expr(n->expr);
  printf("  pop rax\n");
  gen_function_epilogue();
}

static void gen_asm_stmt(Node *n) {
  for (int i = 0; i < n->args->length; i++) {
    Node *str = (Node *)vec_get(n->args, i);
    printf("  %s\n", str->contents);
  }
}

static void gen_countup_stmt(Node *n) {
  int fin_label = label++;

  // initialize
  gen_lval(n->expr);
  gen_expr(n->from);
  printf("  pop rdi\n");
  printf("  pop rax\n");

  store_reg_using_reg("rax", "rdi");

  // in loop
  printf(".Lstart%d:\n", fin_label);

  // check whether condition is satisfied
  gen_expr(n->expr);
  gen_expr(n->to);
  printf("  pop rdi\n");
  printf("  pop rax\n");
  printf("  cmp rax, rdi\n");
  printf("  je .Lend%d\n", fin_label);

  gen_stmts_in_vec(n->body);

  // increment
  gen_lval(n->expr);
  printf("  pop rax\n");
  printf("  mov rdi, [rax]\n");
  printf("  inc rdi\n");

  store_reg_using_reg("rax", "rdi");

  printf("  jmp .Lstart%d\n", fin_label);
  printf(".Lend%d:\n", fin_label);
}

static void gen_expr(Node *n) {
  switch (n->kind) {
  case ND_ADD:
  case ND_SUB:
  case ND_MUL:
  case ND_DIV:
    gen_binary_expr(n);
    break;
  case ND_NEG:
    gen_unary_expr(n);
    break;
  case ND_INTLIT:
    printf("  push %d\n", n->int_value);
    break;
  case ND_STRLIT:
    printf("  push offset .LS%d\n", n->str_n);
    break;
  case ND_CALL:
    gen_exprs_in_vec(n->args);

    for (int i = 0; i < n->args->length; i++) {
      char *reg = caller_regs64[i];
      printf("  pop %s\n", reg);
    }

    printf("  call %s\n", find_library_api(n->id_name));
    printf("  push rax\n");
    break;
  case ND_IDENT:
    gen_lval(n);
    printf("  pop rax\n");

    // get value from address
    printf("  mov rax, [rax]\n");
    printf("  push rax\n");
    break;
  case ND_ASSIGN:
    gen_lval(n->left);
    gen_expr(n->right);
    printf("  pop rdi\n");
    printf("  pop rax\n");

    store_reg_using_reg("rax", "rdi");
    printf("  push rdi\n");
    break;
  case ND_IF:
    if (n->alter) {
      gen_if_else_expr(n);
    } else {
      gen_if_expr(n);
    }
    break;
  case ND_NOP:
    break;
  default:
    fprintf(stderr, "unexpected NodeKind in gen_expr()\n");
    break;
  }
}

static void gen_if_expr(Node *n) {
  gen_expr(n->expr);
  int fin_label = label++;

  printf("  pop rax\n");
  printf("  cmp rax, 0\n");
  printf("  je .Lend%d\n", fin_label);

  gen_stmts_in_vec(n->body);
  printf(".Lend%d:\n", fin_label);
}

static void gen_if_else_expr(Node *n) {
  gen_expr(n->expr);
  int fin_label = label++;

  printf("  pop rax\n");
  printf("  cmp rax, 0\n");
  printf("  je .Lelse%d\n", fin_label);

  gen_stmts_in_vec(n->body);

  printf("  jmp .Lend%d\n", fin_label);

  printf(".Lelse%d:\n", fin_label);

  gen_stmts_in_vec(n->alter);
  printf(".Lend%d:\n", fin_label);
}

static void gen_lval(Node *n) {
  Variable *lvar = NULL;
  switch (n->kind) {
  case ND_IDENT:
    if ((lvar = find_lvar(this_func, n->id_name->name)) == NULL)
      fprintf(stderr, "not found such a variable -> %s\n", n->id_name->name);

    printf("  mov rax, rbp\n");
    printf("  sub rax, %d\n", lvar->offset);
    // if (node->type->kind == T_ADDR) {
    // lea_reg_to_mem("rax", "rax");
    // }
    printf("  push rax\n");
    break;
  default: {
    fprintf(stderr, "unexpected node\n");
  } break;
  }
  return;
}

static void gen_binary_expr(Node *n) {
  // 1. 左右子ノードをコンパイル
  gen_expr(n->left);
  gen_expr(n->right);

  // 2. 演算に必要なオペランドをレジスタに取り出す
  printf("  pop rdi\n");
  printf("  pop rax\n");

  // 3. 各演算に対応する命令でレジスタ操作
  switch (n->kind) {
  case ND_ADD:
  case ND_SUB:
  case ND_MUL:
  case ND_DIV:
    gen_base_op_expr(n->kind);
    break;
  default:
    break;
  }

  // 4. raxに格納された計算結果をスタックに格納
  printf("  push rax\n");
}

static void gen_unary_expr(Node *n) {
  // 1. 左子ノードをコンパイル
  gen_expr(n->left);

  // 2. 演算に必要なオペランドをレジスタに取り出す
  printf("  pop rax\n");

  // 3. 各演算に対応する命令でレジスタ操作
  switch (n->kind) {
  case ND_NEG:
    printf("  neg rax\n");
    break;
  default:
    break;
  }

  // 4. raxに格納された計算結果をスタックに格納
  printf("  push rax\n");
}

static void gen_base_op_expr(NodeKind kind) {
  switch (kind) {
  case ND_ADD:
    printf("  add rax, rdi\n");
    break;
  case ND_SUB:
    printf("  sub rax, rdi\n");
    break;
  case ND_MUL:
    printf("  imul rax, rdi\n");
    break;
  case ND_DIV:
    printf("  cqo\n");
    printf("  idiv rdi\n");
    break;
  default:
    break;
  }
}

static void gen_function_prologue(char *name, uint32_t offset) {
  // symbol
  printf(".global %s\n", name);
  printf("%s:\n", name);

  // save rbp
  printf("  push rbp\n");
  printf("  mov rbp, rsp\n");

  // allocating memory area for auto-var
  //
  if (offset != 0) {
    offset += 7;
    offset &= ~7;
    printf("  sub rsp, %d\n", offset);
  }
}

static void gen_function_epilogue(void) {
  printf("  mov rsp, rbp\n");
  printf("  pop rbp\n");
  printf("  ret\n");
}

static void store_reg(uint32_t offset, char *reg) {
  printf("  mov -%d[rbp], %s\n", offset, reg);
}

static void store_reg_using_reg(char *addr_reg, char *val_reg) {
  printf("  mov [%s], %s\n", addr_reg, val_reg);
}

static char *find_library_api(IdentName *id_name) {
  IdentName *iter = id_name;
  char *name = iter->name;
  while (iter->next) {
    iter = iter->next;
    name = iter->name;
  }
  return name;
}

static void gen_stmts_in_vec(Vector *v) {
  for (int i = 0; i < v->length; i++) {
    Node *st = (Node *)vec_get(v, i);
    gen_stmt(st);
  }
}

static void gen_exprs_in_vec(Vector *v) {
  for (int i = 0; i < v->length; i++) {
    Node *st = (Node *)vec_get(v, i);
    gen_expr(st);
  }
}

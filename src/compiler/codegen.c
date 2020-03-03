#include "ast.h"
#include "variable.h"
#include "vector.h"

static void gen_lval(Function *func, Node *n);
static void gen_stmt(Function *func, Node *n);
static void gen_expr(Function *func, Node *n);
static void gen_base_op_expr(NodeKind kind);
static void gen_binary_expr(Function *func, Node *n);
static void gen_unary_expr(Function *func, Node *n);

void gen_x64(Function *func) {
  printf(".intel_syntax noprefix\n");
  printf(".global %s\n", func->name);
  printf("%s:\n", func->name);
  printf("  push rbp\n");
  printf("  mov rbp, rsp\n");
  if (func->stack_offset != 0) {
    func->stack_offset += 7;
    func->stack_offset &= ~7;
    printf("  sub rsp, %d\n", func->stack_offset);
  }
  for (int i = 0; i < func->stmts->length; i++) {
    Node *stmt = get_statement(func, i);
    gen_stmt(func, stmt);
  }
}

static void gen_stmt(Function *func, Node *n) {
  switch (n->kind) {
    case ND_RETURN:
      gen_expr(func, n->expr);
      printf("  pop rax\n");
      printf("  mov rsp, rbp\n");
      printf("  pop rbp\n");
      printf("  ret\n");
      break;
    default:
      // expression-statementとする
      gen_expr(func, n);
      break;
  }
}
static void gen_expr(Function *func, Node *n) {
  switch (n->kind) {
    case ND_ADD:
    case ND_SUB:
    case ND_MUL:
    case ND_DIV:
      gen_binary_expr(func, n);
      break;
    case ND_NEG:
      gen_unary_expr(func, n);
      break;
    case ND_INTLIT:
      printf("  push %d\n", n->int_value);
      break;
    case ND_IDENT:
      gen_lval(func, n);
      printf("  pop rax\n");
      printf("  mov rax, [rax]\n");
      printf("  push rax\n");
      break;
    case ND_ASSIGN:
      gen_lval(func, n->left);
      gen_expr(func, n->right);
      printf("  pop rdi\n");
      printf("  pop rax\n");
      printf("  mov [rax], rdi\n");
      printf("  push rdi\n");
      break;
    case ND_NOP:
      break;
    default:
      fprintf(stderr, "unexpected NodeKind in gen_expr()\n");
      break;
  }
}

static void gen_lval(Function *func, Node *n) {
  Variable *lvar = NULL;
  switch (n->kind) {
    case ND_IDENT:
      if ((lvar = find_lvar(func, n->name)) == NULL)
        fprintf(stderr, "not found such a variable -> %s\n", n->name);

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

static void gen_binary_expr(Function *func, Node *n) {
  // 1. 左右子ノードをコンパイル
  gen_expr(func, n->left);
  gen_expr(func, n->right);

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

static void gen_unary_expr(Function *func, Node *n) {
  // 1. 左子ノードをコンパイル
  gen_expr(func, n->left);

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

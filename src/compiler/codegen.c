#include "ast.h"

static void gen_expr(Node *n);
static void gen_base_op_expr(NodeKind kind);
static void gen_binary_expr(Node *n);

void gen_x64(Node *top_node) {
  printf(".intel_syntax noprefix\n");
  printf(".global main\n");
  printf("main:\n");
  gen_expr(top_node);
  printf("  pop rax\n");
  printf("  ret\n");
}

static void gen_expr(Node *n) {
  switch (n->kind) {
    case ND_ADD:
    case ND_SUB:
      gen_binary_expr(n);
      break;
    case ND_INTLIT:
      printf("  push %d\n", n->int_value);
      break;
  }
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
      gen_base_op_expr(n->kind);
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
    default:
      break;
  }
}

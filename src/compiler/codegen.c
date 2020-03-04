#include "ast.h"
#include "variable.h"
#include "vector.h"

static void gen_lval(Node *n);
static void gen_stmt(Node *n);
static void gen_expr(Node *n);
static void gen_base_op_expr(NodeKind kind);
static void gen_binary_expr(Node *n);
static void gen_unary_expr(Node *n);

static Function *this_func;
static int label = 0;

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

  this_func = func;
  for (int i = 0; i < func->stmts->length; i++) {
    Node *stmt = get_statement(func, i);
    gen_stmt(stmt);
  }
}

static void gen_stmt(Node *n) {
  switch (n->kind) {
    case ND_RETURN:
      gen_expr(n->expr);
      printf("  pop rax\n");
      printf("  mov rsp, rbp\n");
      printf("  pop rbp\n");
      printf("  ret\n");
      break;
    case ND_IFRET:
      gen_expr(n->expr);
      break;
    default:
      // expression-statementとする
      gen_expr(n);
      break;
  }
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
    case ND_IDENT:
      gen_lval(n);
      printf("  pop rax\n");
      printf("  mov rax, [rax]\n");
      printf("  push rax\n");
      break;
    case ND_ASSIGN:
      gen_lval(n->left);
      gen_expr(n->right);
      printf("  pop rdi\n");
      printf("  pop rax\n");
      printf("  mov [rax], rdi\n");
      printf("  push rdi\n");
      break;
    case ND_IF:
      gen_expr(n->expr);
      int fin_label = label++;

      printf("  pop rax\n");
      printf("  cmp rax, 0\n");
      printf("  je .Lend%d\n", fin_label);

      for (int i = 0; i < n->body->length; i++) {
        Node *st = (Node *)vec_get(n->body, i);
        gen_stmt(st);
      }
      printf(".Lend%d:\n", fin_label);
      break;
    case ND_NOP:
      break;
    default:
      fprintf(stderr, "unexpected NodeKind in gen_expr()\n");
      break;
  }
}

static void gen_lval(Node *n) {
  Variable *lvar = NULL;
  switch (n->kind) {
    case ND_IDENT:
      if ((lvar = find_lvar(this_func, n->name)) == NULL)
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

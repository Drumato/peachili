#include "ast.h"

#include "base.h"

static Node *init_node(NodeKind kind);
static void dealloc_node(Node *n);
static void debug_binary(char *operator, Node *n);
static void debug_unary(char *operator, Node *n);
static void debug(Node *n);

void dealloc_function(Function *func) {
  free(func->name);
  func->name = NULL;
  free(func->return_type);
  func->return_type = NULL;

  dealloc_node(func->stmt);
}

static void dealloc_node(Node *n) {
  switch (n->kind) {
    case ND_ADD:
    case ND_SUB:
    case ND_MUL:
    case ND_DIV:
      free(n->left);
      n->left = NULL;
      free(n->right);
      n->right = NULL;
      break;
    case ND_NEG:
      free(n->left);
      n->left = NULL;
      break;
    case ND_RETURN:
      free(n->expr);
      n->expr = NULL;
      break;
    default:
      break;
  }
  free(n);
}

// コンストラクタ
Function *new_function(char *name, Node *stmt, AGType *ret_type, uint32_t col, uint32_t row) {
  Function *func = (Function *)calloc(1, sizeof(Function));

  int length = strlen(name);
  func->name = (char *)calloc(length, sizeof(char));
  strncpy(func->name, name, length);
  func->name[length] = 0;

  func->stmt        = stmt;
  func->return_type = ret_type;
  func->col         = col;
  func->row         = row;
  return func;
}

Node *new_return(Node *expr, uint32_t col, uint32_t row) {
  Node *n = init_node(ND_RETURN);
  n->expr = expr;
  n->col  = col;
  n->row  = row;
  return n;
}
Node *new_binary_node(NodeKind kind, Node *lhs, Node *rhs, uint32_t col, uint32_t row) {
  Node *n  = init_node(kind);
  n->left  = lhs;
  n->right = rhs;
  n->col   = col;
  n->row   = row;
  return n;
}

Node *new_unary_node(NodeKind kind, Node *inner, uint32_t col, uint32_t row) {
  Node *n = init_node(kind);
  n->left = inner;
  n->col  = col;
  n->row  = row;
  return n;
}

Node *new_intlit_node(int value, uint32_t col, uint32_t row) {
  Node *n      = init_node(ND_INTLIT);
  n->int_value = value;
  n->col       = col;
  n->row       = row;
  return n;
}

void debug_func_to_stderr(bool verbose, Function *func) {
  if (verbose) {
    fprintf(stderr, "++++++++ debug-ast ++++++++\n");
    fprintf(stderr, "func %s() { \n", func->name);
    fprintf(stderr, "\t");
    debug(func->stmt);
    fprintf(stderr, "}\n");
    fprintf(stderr, "\n\n");
  }
}

static Node *init_node(NodeKind kind) {
  Node *n = (Node *)calloc(1, sizeof(Node));
  n->kind = kind;
  return n;
}

static void debug(Node *n) {
  switch (n->kind) {
    case ND_INTLIT:
      fprintf(stderr, "%d", n->int_value);
      break;
    case ND_ADD:
      debug_binary("ADD", n);
      break;
    case ND_SUB:
      debug_binary("SUB", n);
      break;
    case ND_MUL:
      debug_binary("MUL", n);
      break;
    case ND_DIV:
      debug_binary("DIV", n);
      break;
    case ND_NEG:
      debug_unary("NEG", n);
      break;
    case ND_RETURN:
      fprintf(stderr, "return ");
      debug(n->expr);
      fprintf(stderr, ";\n");
      break;
  }
}

static void debug_unary(char *operator, Node *n) {
  fprintf(stderr, "%s(", operator);
  debug(n->left);
  fprintf(stderr, ")");
}

static void debug_binary(char *operator, Node *n) {
  fprintf(stderr, "%s(", operator);
  debug(n->left);
  fprintf(stderr, ", ");
  debug(n->right);
  fprintf(stderr, ")");
}

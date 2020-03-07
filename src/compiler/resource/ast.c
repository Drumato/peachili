#include "ast.h"

#include "agtype.h"
#include "base.h"
#include "variable.h"
#include "vector.h"

static Node *init_node(NodeKind kind);
static void dealloc_node(Node *n);
static void debug_binary(char *operator, Node *n);
static void debug_unary(char *operator, Node *n);
static void debug(Node *n);

// related Function
Node *get_statement(Function *func, int idx) { return (Node *)vec_get(func->stmts, idx); }
void put_statement(Function *func, Node *n) { vec_push(func->stmts, (void *)n); }
Variable *get_local_var(Function *func, int idx) { return (Variable *)vec_get(func->locals, idx); }
void put_local_var(Function *func, Variable *var) { vec_push(func->locals, (void *)var); }

Variable *find_lvar(Function *func, char *name) {
  for (int i = 0; i < func->locals->length; i++) {
    Variable *var = get_local_var(func, i);
    if (!strncmp(var->name, name, strlen(name))) {
      return var;
    }
  }

  return NULL;
}

void dealloc_function(Function *func) {
  free(func->name);
  func->name = NULL;
  free(func->return_type);
  func->return_type = NULL;

  for (int i = 0; i < func->stmts->length; i++) {
    Node *n = get_statement(func, i);
    dealloc_node(n);
  }

  for (int i = 0; i < func->locals->length; i++) {
    Variable *var = get_local_var(func, i);
    free(var->name);
    free(var);
  }
}

static void dealloc_node(Node *n) {
  switch (n->kind) {
    case ND_ADD:
    case ND_SUB:
    case ND_MUL:
    case ND_DIV:
    case ND_ASSIGN:
      free(n->left);
      n->left = NULL;
      free(n->right);
      n->right = NULL;
      break;
    case ND_NEG:
      free(n->left);
      n->left = NULL;
      break;
    case ND_IFRET:
    case ND_RETURN:
      free(n->expr);
      n->expr = NULL;
      break;
    case ND_COUNTUP:
      free(n->expr);
      n->expr = NULL;
      free(n->from);
      n->from = NULL;
      free(n->to);
      n->to = NULL;
      free(n->body);
      n->body = NULL;
      break;
    case ND_IDENT:
      free(n->name);
      n->name = NULL;
      break;
    case ND_IF:
      free(n->expr);
      n->expr = NULL;
      free(n->body);
      n->body = NULL;
      if (!n->alter) {
        free(n->alter);
        n->alter = NULL;
      }
      break;
    default:
      break;
  }
  free(n);
}

// コンストラクタ
Function *new_function(char *name, AGType *ret_type, uint32_t col, uint32_t row) {
  Function *func = (Function *)calloc(1, sizeof(Function));

  int length = strlen(name);
  func->name = (char *)calloc(length, sizeof(char));
  strncpy(func->name, name, length);
  func->name[length] = 0;

  func->stmts       = new_vec();
  func->locals      = new_vec();
  func->return_type = ret_type;
  func->col         = col;
  func->row         = row;
  return func;
}

Node *new_nop(void) {
  Node *n = init_node(ND_NOP);
  return n;
}

Node *new_if(Node *cond, Vector *stmts, Vector *alter, uint32_t col, uint32_t row) {
  Node *n  = init_node(ND_IF);
  n->expr  = cond;
  n->body  = stmts;
  n->alter = alter;
  n->col   = col;
  n->row   = row;
  return n;
}

Node *new_return(Node *expr, uint32_t col, uint32_t row) {
  Node *n = init_node(ND_RETURN);
  n->expr = expr;
  n->col  = col;
  n->row  = row;
  return n;
}
Node *new_countup(Node *lvar, Node *start, Node *end, struct Vector *stmts, uint32_t col,
                  uint32_t row) {
  Node *n = init_node(ND_COUNTUP);
  n->expr = lvar;
  n->body = stmts;
  n->from = start;
  n->to   = end;
  n->col  = col;
  n->row  = row;
  return n;
}
Node *new_ifret(Node *expr, uint32_t col, uint32_t row) {
  Node *n = init_node(ND_IFRET);
  n->expr = expr;
  n->col  = col;
  n->row  = row;
  return n;
}
Node *new_assign(Node *lvar, Node *expr, uint32_t col, uint32_t row) {
  Node *n  = init_node(ND_ASSIGN);
  n->left  = lvar;
  n->right = expr;
  n->col   = col;
  n->row   = row;
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

Node *new_ident_node(char *name, uint32_t col, uint32_t row) {
  Node *n = init_node(ND_IDENT);
  n->col  = col;
  n->row  = row;

  int length = strlen(name);
  n->name    = (char *)calloc(length, sizeof(char));
  strncpy(n->name, name, length);
  n->name[length] = 0;

  return n;
}

void debug_func_to_stderr(bool verbose, Function *func) {
  if (verbose) {
    fprintf(stderr, "++++++++ debug-ast ++++++++\n");
    fprintf(stderr, "func %s() ", func->name);
    dump_agtype(func->return_type);
    fprintf(stderr, " {\n");
    for (int i = 0; i < func->stmts->length; i++) {
      Node *stmt = get_statement(func, i);
      fprintf(stderr, "\t");
      debug(stmt);
    }
    fprintf(stderr, "}\n");
    fprintf(stderr, "local-var definitions in %s() \n", func->name);
    for (int i = 0; i < func->locals->length; i++) {
      Variable *var = get_local_var(func, i);
      fprintf(stderr, "\t%d: %s ", i, var->name);
      dump_agtype(var->type);
      fprintf(stderr, "\n");
    }
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
    case ND_COUNTUP:
      fprintf(stderr, "countup");
      break;
    case ND_IF:
      fprintf(stderr, "if (");
      debug(n->expr);
      fprintf(stderr, ") {\n");
      for (int i = 0; i < n->body->length; i++) {
        Node *stmt = vec_get(n->body, i);
        fprintf(stderr, "\t");
        debug(stmt);
      }

      if (n->alter) {
        fprintf(stderr, " else {\n");
        for (int i = 0; i < n->alter->length; i++) {
          Node *stmt = vec_get(n->alter, i);
          fprintf(stderr, "\t");
          debug(stmt);
        }
      }

      fprintf(stderr, "\t}\n");
      break;
    case ND_IFRET:
      fprintf(stderr, "ifret ");
      debug(n->expr);
      fprintf(stderr, ";\n");
      break;
    case ND_RETURN:
      fprintf(stderr, "return ");
      debug(n->expr);
      fprintf(stderr, ";\n");
      break;
    default:
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

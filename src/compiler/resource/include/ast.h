#include "base.h"
#include "token.h"

typedef struct Node Node;
typedef struct {
  char *name;
  struct AGType *return_type;
  struct Vector *stmts;
  uint32_t col;
  uint32_t row;
} Function;

Function *new_function(char *name, struct AGType *ret_type, uint32_t col, uint32_t row);

typedef enum {
  ND_INTLIT,

  // binary-operation
  ND_ADD,
  ND_SUB,
  ND_MUL,
  ND_DIV,

  // unary-operation
  ND_NEG,

  // statement
  ND_RETURN,
} NodeKind;

struct Node {
  // general
  NodeKind kind;
  uint32_t col;
  uint32_t row;

  // for expressions
  int int_value;
  Node *left;
  Node *right;

  // for statements
  Node *expr;
};

Node *new_return(Node *expr, uint32_t col, uint32_t row);
Node *new_binary_node(NodeKind kind, Node *lhs, Node *rhs, uint32_t col, uint32_t row);
Node *new_unary_node(NodeKind kind, Node *inner, uint32_t col, uint32_t row);
Node *new_intlit_node(int length, uint32_t col, uint32_t row);
void debug_func_to_stderr(bool verbose, Function *func);
void dealloc_function(Function *func);

Node *vec_get_as_a_node(struct Vector *vec, int idx);
void push_node_into_vec(struct Vector *vec, Node *n);

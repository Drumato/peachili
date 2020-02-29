#include "base.h"

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

typedef struct Node Node;
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
void debug_ast_to_stderr(bool verbose, Node *top_node);
void dealloc_node(Node *n);

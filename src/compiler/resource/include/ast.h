#include "base.h"

typedef enum {
  ND_INTLIT,
  ND_ADD,
  ND_SUB,
} NodeKind;

typedef struct Node Node;
struct Node {
  NodeKind kind;
  Node *left;
  Node *right;
  int int_value;
  uint32_t col;
  uint32_t row;
};

Node *new_binary_node(NodeKind kind, Node *lhs, Node *rhs, uint32_t col, uint32_t row);
Node *new_intlit_node(int length, uint32_t col, uint32_t row);
void debug_ast_to_stderr(bool verbose, Node *top_node);
void dealloc_node(Node *n);

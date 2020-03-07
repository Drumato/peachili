#include "base.h"
#include "token.h"

typedef struct Node Node;
typedef struct {
  char *name;
  struct AGType *return_type;
  struct Vector *stmts;
  uint32_t col;
  uint32_t row;
  struct Vector *locals;
  int stack_offset;
} Function;

Function *new_function(char *name, struct AGType *ret_type, uint32_t col, uint32_t row);

typedef enum {
  ND_INTLIT,
  ND_IDENT,

  // binary-operation
  ND_ADD,
  ND_SUB,
  ND_MUL,
  ND_DIV,

  // unary-operation
  ND_NEG,

  // statement
  ND_RETURN,
  ND_IFRET,
  ND_COUNTUP,

  // etc
  ND_ASSIGN,
  ND_IF,
  ND_NOP,
} NodeKind;

struct Node {
  // general
  NodeKind kind;
  uint32_t col;
  uint32_t row;

  // for identifier
  char *name;

  // for expressions
  int int_value;
  Node *left;
  Node *right;
  struct Vector *body;
  struct Vector *alter;

  // for statements
  Node *expr;
  Node *from;
  Node *to;
};

Node *new_return(Node *expr, uint32_t col, uint32_t row);
Node *new_ifret(Node *expr, uint32_t col, uint32_t row);
Node *new_countup(Node *lvar, Node *start, Node *end, struct Vector *stmts, uint32_t col,
                  uint32_t row);
Node *new_if(Node *cond, struct Vector *stmts, struct Vector *alter, uint32_t col, uint32_t row);
Node *new_nop(void);
Node *new_assign(Node *lvar, Node *expr, uint32_t col, uint32_t row);
Node *new_binary_node(NodeKind kind, Node *lhs, Node *rhs, uint32_t col, uint32_t row);
Node *new_unary_node(NodeKind kind, Node *inner, uint32_t col, uint32_t row);
Node *new_intlit_node(int length, uint32_t col, uint32_t row);
Node *new_ident_node(char *name, uint32_t col, uint32_t row);
void debug_func_to_stderr(bool verbose, Function *func);
void dealloc_function(Function *func);
struct Variable *find_lvar(Function *func, char *name);

Node *get_statement(Function *func, int idx);
void put_statement(Function *func, Node *n);
struct Variable *get_local_var(Function *func, int idx);
void put_local_var(Function *func, struct Variable *var);

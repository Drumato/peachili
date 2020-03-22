#include "base.h"
#include "token.h"

typedef enum {
  FN_DEFINED,
  FN_BUILTIN,
} FuncKind;

typedef struct Node Node;
typedef struct {
  char *name;
  struct AGType *return_type;
  struct Vector *stmts;
  uint32_t col;
  uint32_t row;
  struct Vector *locals;
  struct Vector *args;
  uint32_t stack_offset;
  FuncKind kind;
} Function;

Function *new_function(char *name, struct AGType *ret_type, uint32_t col,
                       uint32_t row);

typedef enum {
  ND_INTLIT,
  ND_STRLIT,
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
  ND_CALL,
  ND_NOP,
} NodeKind;

typedef struct IdentName IdentName;
struct IdentName {
  char *name;
  IdentName *next;
};

IdentName *new_ident_name(char *name, IdentName *next);
IdentName *append_ident_name(char *name, IdentName **cur);

struct Node {
  // general
  NodeKind kind;
  uint32_t col;
  uint32_t row;

  // for identifier
  IdentName *id_name;

  // for expressions
  int int_value;
  char *contents;
  uint32_t str_n;
  Node *left;
  Node *right;
  struct Vector *body;
  struct Vector *alter;
  struct Vector *args;

  // for statements
  Node *expr;
  Node *from;
  Node *to;
};

Node *new_return(Node *expr, uint32_t col, uint32_t row);
Node *new_call(IdentName *id_name, struct Vector *args, uint32_t col,
               uint32_t row);
Node *new_ifret(Node *expr, uint32_t col, uint32_t row);
Node *new_countup(Node *lvar, Node *start, Node *end, struct Vector *stmts,
                  uint32_t col, uint32_t row);
Node *new_if(Node *cond, struct Vector *stmts, struct Vector *alter,
             uint32_t col, uint32_t row);
Node *new_nop(void);
Node *new_assign(Node *lvar, Node *expr, uint32_t col, uint32_t row);
Node *new_binary_node(NodeKind kind, Node *lhs, Node *rhs, uint32_t col,
                      uint32_t row);
Node *new_unary_node(NodeKind kind, Node *inner, uint32_t col, uint32_t row);
Node *new_intlit_node(int length, uint32_t col, uint32_t row);
Node *new_strlit_node(char *str, uint32_t str_n, uint32_t col, uint32_t row);
Node *new_ident_node(IdentName *id_name, uint32_t col, uint32_t row);
void debug_func_to_stderr(bool verbose, Function *func);
void dealloc_function(Function *func);
struct Variable *find_lvar(Function *func, char *name);

Node *get_statement(Function *func, int idx);
void put_statement(Function *func, Node *n);
struct Variable *get_local_var(Function *func, int idx);
void put_local_var(Function *func, struct Variable *var);

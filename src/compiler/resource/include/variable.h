typedef enum { VAR_LOCAL } VarKind;

struct Variable {
  VarKind kind;
  struct AGType *type;
  char *name;
  int offset;
};
typedef struct Variable Variable;

Variable *new_local_var(char *name, struct AGType *type);

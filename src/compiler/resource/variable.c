#include "variable.h"

#include "agtype.h"
#include "vector.h"

Variable *new_local_var(char *name, AGType *type) {
  Variable *local = (Variable *)calloc(1, sizeof(Variable));
  local->kind     = VAR_LOCAL;
  local->type     = type;

  int length  = strlen(name);
  local->name = (char *)calloc(length, sizeof(char));
  strncpy(local->name, name, length);
  local->name[length] = 0;

  return local;
}

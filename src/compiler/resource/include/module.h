#include "base.h"

typedef enum {
  MD_PRIMARY,
  MD_EXTERNAL,
} ModuleKind;

struct Module {
  char *file_path;
  ModuleKind kind;
  struct Vector *functions;
  struct Vector *requires;
  struct Vector *used;
  bool visited;
};

typedef struct Module Module;

Module *find_required_mod(Module *base, char *name);
Module *new_module(ModuleKind kind, char *file_path);
bool function_is_used(Module *mod, char *name);

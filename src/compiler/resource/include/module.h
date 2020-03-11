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
  bool visited;
};

typedef struct Module Module;

Module *new_module(ModuleKind kind, char *file_path);

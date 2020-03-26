#include "module.h"

#include "util.h"
#include "vector.h"

#define FILE_SUFFIX_LEN 3

Module *new_module(ModuleKind kind, char *file_path) {
  Module *mod = (Module *)calloc(1, sizeof(Module));
  mod->kind = kind;
  mod->functions = new_vec();
  mod->requires = new_vec();
  mod->strings = new_vec();
  mod->subs = new_vec();
  mod->visited = false;
  mod->file_path = file_path;
  return mod;
}
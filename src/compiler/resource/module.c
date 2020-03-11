#include "module.h"

#include "vector.h"

Module *new_module(ModuleKind kind, char *file_path) {
  Module *mod    = (Module *)calloc(1, sizeof(Module));
  mod->kind      = kind;
  mod->functions = new_vec();
  mod->requires  = new_vec();
  mod->visited   = false;

  int length     = strlen(file_path);
  mod->file_path = (char *)calloc(length, sizeof(char));
  strncpy(mod->file_path, file_path, length);
  mod->file_path[length] = 0;
  return mod;
}

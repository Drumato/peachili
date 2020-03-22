#include "module.h"

#include "util.h"
#include "vector.h"

#define FILE_SUFFIX_LEN 3

Module *find_required_mod(Module *base, char *name) {
  for (int i = 0; i < base->requires->length; i++) {
    Module *req_mod = (Module *)vec_get(base->requires, i);

    // strtok によって値が変わってしまうのでコピーを渡す
    char *req_mod_file_path = str_alloc_and_copy(req_mod->file_path, strlen(req_mod->file_path));
    char *last_path = get_last_path(req_mod_file_path);
    if (!strncmp(last_path, name,
                 strlen(last_path) - FILE_SUFFIX_LEN)) {
      return req_mod;
    }
  }

  return NULL;
}

Module *new_module(ModuleKind kind, char *file_path) {
  Module *mod = (Module *)calloc(1, sizeof(Module));
  mod->kind = kind;
  mod->functions = new_vec();
  mod->requires = new_vec();
  mod->used = new_vec();
  mod->strings = new_vec();
  mod->visited = false;

  mod->file_path = str_alloc_and_copy(file_path, strlen(file_path));
  return mod;
}

bool function_is_used(Module *mod, char *name) {
  for (int i = 0; i < mod->used->length; i++) {
    char *api_name = (char *)vec_get(mod->used, i);
    if (!strncmp(api_name, name, strlen(api_name))) {
      return true;
    }
  }
  return false;
}

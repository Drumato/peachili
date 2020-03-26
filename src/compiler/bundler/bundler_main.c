#include "base.h"
#include "bundler.h"
#include "module.h"
#include "structure.h"
#include "token.h"
#include "util.h"
#include "vector.h"

extern Token *tokenize(char *program);
extern void bundler_parse(Module **mod, Token **top_token);

static void debug_bundler(void);

void bundler_init(DebugOption *debug_opt, char *file_path) {
  char *user_input = get_contents(file_path);
  Token *top_token = tokenize(user_input);

  char *lib_path = getenv("PEACHILI_LIB_PATH");
  lib_path_env = str_alloc_and_copy(lib_path, strlen(lib_path));
  assert(lib_path_env);

  sources_g = new_vec();

  // コンパイル対象は先に追加しておく
  char *mod_path = (char *)calloc(strlen(file_path), sizeof(char));
  strncpy(mod_path, file_path, strlen(file_path));
  file_path[strlen(mod_path)] = '\0';
  Module *main_module = new_module(MD_PRIMARY, mod_path);
  vec_push(sources_g, (void *)main_module);
  bundler_parse(&main_module, &top_token);

  if (debug_opt->dbg_compiler) {
    debug_bundler();
  }

  dealloc_tokens(&top_token);
}

static void debug_bundler(void) {
  fprintf(stderr, "++++++++ debug bundler ++++++++\n");
  fprintf(stderr, "\tenumerate files\n");
  for (int i = 0; i < sources_g->length; i++) {
    Module *m = (Module *)vec_get(sources_g, i);
    fprintf(stderr, "\t\tfile_path -> %s\n", m->file_path);
    fprintf(stderr, "\t\tthe number of defined functions -> %d\n",
            m->functions->length);

    fprintf(stderr, "\t\tsub modules:\n");
    for (int sub_i = 0; sub_i < m->subs->length; sub_i++) {
      Module *sub_mod = (Module *)vec_get(m->subs, sub_i);
      fprintf(stderr, "\t\t\t%s\n", sub_mod->file_path);
    }

    fprintf(stderr, "\t\trequired modules:\n");
    for (int req_i = 0; req_i < m->requires->length; req_i++) {
      Module *req_mod = (Module *)vec_get(m->requires, req_i);
      fprintf(stderr, "\t\t\t%s\n", req_mod->file_path);
    }
  }
}

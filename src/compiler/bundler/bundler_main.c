#include "base.h"
#include "bundler.h"
#include "module.h"
#include "structure.h"
#include "token.h"
#include "util.h"
#include "vector.h"

extern Token *tokenize(char *program);
extern void bundler_parse(Module **mod, Token **top_token);

void bundler_init(DebugOption *debug_opt, char *file_path) {
  char *user_input = get_contents(file_path);
  Token *top_token = tokenize(user_input);

  sources_g = new_vec();

  // コンパイル対象は先に追加しておく
  Module *main_module = new_module(MD_PRIMARY, file_path);

  bundler_parse(&main_module, &top_token);

  if (debug_opt->dbg_compiler) {
    fprintf(stderr, "++++++++ debug bundler ++++++++\n");
    fprintf(stderr, "\tenumerate files\n");
    for (int i = 0; i < sources_g->length; i++) {
      Module *m = (Module *)vec_get(sources_g, i);
      fprintf(stderr, "\t\t%s\n", m->file_path);
    }
  }

  dealloc_tokens(&top_token);
}

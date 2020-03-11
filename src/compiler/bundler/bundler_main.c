#include "base.h"
#include "bundler.h"
#include "module.h"
#include "token.h"
#include "vector.h"

extern char *get_contents(const char *filename);
extern Token *tokenize(char *program);
extern void bundler_parse(Module **mod, Token **top_token);

void bundler_init(char *file_path) {
  char *user_input = get_contents(file_path);
  Token *top_token = tokenize(user_input);

  sources_g = new_vec();

  // コンパイル対象は先に追加しておく
  Module *main_module = new_module(MD_PRIMARY, file_path);
  vec_push(sources_g, main_module);

  bundler_parse(&main_module, &top_token);

  dealloc_tokens(&top_token);
}

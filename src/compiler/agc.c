#include "ast.h"
#include "base.h"
#include "structure.h"

extern Token *tokenize(char *program);
extern Function *parse(Token *top_token);
extern void gen_x64(Function *func);

void compiler_main(int argc, char **argv, DebugOption *debug_opt) {
  if (argc < 2) {
    fprintf(stderr, "invalid arguments.\n usage: %s <source-file>\n", argv[0]);
    exit(1);
  }

  // TODO: 後々ファイル読み込みに変更する
  Token *top_token = tokenize(argv[optind]);

  debug_tokens_to_stderr(debug_opt->verbose, top_token);

  Function *main_func = parse(top_token);

  debug_func_to_stderr(debug_opt->verbose, main_func);

  gen_x64(main_func);
  dealloc_tokens(&top_token);
  dealloc_function(main_func);
}

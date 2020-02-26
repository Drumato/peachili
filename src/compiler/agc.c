#include "base.h"
#include "structure.h"
#include "token.h"

extern Token *lexing(char *program);
void compiler_main(int argc, char **argv, DebugOption *debug_opt) {
  if (argc < 2) {
    fprintf(stderr, "invalid arguments.\n usage: %s <source-file>\n", argv[0]);
  }

  if (debug_opt->verbose) {
    fprintf(stderr, "start compiling with verbosity...\n");
  }

  // TODO: 後々ファイル読み込みに変更する
  Token *top_token = lexing(argv[optind]);

  dump_tokens_to_stderr_if_verbose(debug_opt->verbose, top_token);

  dealloc_token(&top_token);
}

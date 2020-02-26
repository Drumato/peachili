#include "ast.h"
#include "base.h"
#include "structure.h"
#include "token.h"

extern Token *tokenize(char *program);
extern Node *parse(Token *top_token);

void compiler_main(int argc, char **argv, DebugOption *debug_opt) {
  if (argc < 2) {
    fprintf(stderr, "invalid arguments.\n usage: %s <source-file>\n", argv[0]);
    exit(1);
  }

  // TODO: 後々ファイル読み込みに変更する
  Token *top_token = tokenize(argv[optind]);

  debug_tokens_to_stderr(debug_opt->verbose, top_token);

  Node *top_node = parse(top_token);
  dealloc_tokens(&top_token);

  debug_ast_to_stderr(debug_opt->verbose, top_node);

  dealloc_node(top_node);
}

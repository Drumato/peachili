#include "ast.h"
#include "base.h"
#include "structure.h"
#include "vector.h"

extern char *get_contents(const char *filename);
extern Token *tokenize(char *program);
extern Vector *parse(Token *top_token);
extern void type_check(Function **func);
extern void allocate_stack_frame(Function **func);
extern void gen_x64(Vector *functions);

void compiler_main(int argc, char **argv, DebugOption *debug_opt) {
  if (argc < 2) {
    fprintf(stderr, "invalid arguments.\n usage: %s <source-file>\n", argv[0]);
    exit(1);
  }

  char *user_input = get_contents(argv[optind]);

  // TODO: 後々ファイル読み込みに変更する
  Token *top_token = tokenize(user_input);

  debug_tokens_to_stderr(debug_opt->dbg_compiler, top_token);

  Vector *functions = parse(top_token);
  dealloc_tokens(&top_token);

  for (int i = 0; i < functions->length; i++) {
    Function *iter_func = (Function *)vec_get(functions, i);
    debug_func_to_stderr(debug_opt->dbg_compiler, iter_func);
    // 型検査 && 構文検査
    type_check(&iter_func);
    allocate_stack_frame(&iter_func);
  }

  gen_x64(functions);

  for (int i = 0; i < functions->length; i++) {
    Function *iter_func = (Function *)vec_get(functions, i);

    dealloc_function(iter_func);
  }
}

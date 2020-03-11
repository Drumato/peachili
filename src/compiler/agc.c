#include "ast.h"
#include "base.h"
#include "structure.h"
#include "vector.h"

extern char *get_contents(const char *filename);
extern Token *tokenize(char *program);
extern Vector *parse(Token *top_token);
extern void type_check(Vector **functions);
extern void allocate_stack_frame(Vector **functions);
extern void gen_x64(Vector *functions);

void compiler_main(int argc, char **argv, DebugOption *debug_opt) {
  if (argc < 2) {
    fprintf(stderr, "invalid arguments.\n usage: %s <source-file>\n", argv[0]);
    exit(1);
  }

  char *user_input = get_contents(argv[optind]);

  // step.1 tokenize
  Token *top_token = tokenize(user_input);

  debug_tokens_to_stderr(debug_opt->dbg_compiler, top_token);

  // step.2 parse
  Vector *functions = parse(top_token);
  dealloc_tokens(&top_token);

  // step.3 typecheck and allocating_stack
  for (int i = 0; i < functions->length; i++) {
    Function *iter_func = (Function *)vec_get(functions, i);
    debug_func_to_stderr(debug_opt->dbg_compiler, iter_func);
  }

  type_check(&functions);
  allocate_stack_frame(&functions);

  // step.4 code-generating finally
  gen_x64(functions);

  for (int i = 0; i < functions->length; i++) {
    Function *iter_func = (Function *)vec_get(functions, i);

    dealloc_function(iter_func);
  }
}

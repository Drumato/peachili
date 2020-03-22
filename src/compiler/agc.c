#include "ast.h"
#include "base.h"
#include "bundler.h"
#include "module.h"
#include "structure.h"
#include "util.h"
#include "vector.h"

extern void bundler_init(DebugOption *debug_opt, char *file_path);

extern Token *tokenize(char *program);

extern Vector *parse(Token *top_token, int source_i);

extern void type_check(Vector **functions);

extern void allocate_stack_frame(Vector **functions);

extern void gen_x64_primary(Vector *functions);

extern void gen_x64_external(Module *mod);
extern void gen_x64_strlit(Module *mod);
static void proc_frontend(int source_i, DebugOption *debug_opt, Module **mod);
static void proc_backend(Module **mod);

void compiler_main(int argc, char **argv, DebugOption *debug_opt) {
  if (argc < 2) {
    fprintf(stderr, "invalid arguments.\n usage: %s <source-file>\n", argv[0]);
    exit(1);
  }
  char *file_path = argv[optind];

  // *********************
  // *      Bundler      *
  // *********************

  bundler_init(debug_opt, file_path);

  // *********************
  // *      Compiler     *
  // *********************

  // 最初，すべてのソースに対しフロントエンド処理を終わらせる．
  for (int source_i = 0; source_i < sources_g->length; source_i++) {
    Module *mod = (Module *)vec_get(sources_g, source_i);
    proc_frontend(source_i, debug_opt, &mod);
  }

  // TODO: 実際に外部ファイルに識別子が宣言されているかチェックする意味解析

  // step.4 code-generating finally
  printf(".intel_syntax noprefix\n");

  for (int source_i = 0; source_i < sources_g->length; source_i++) {
    Module *mod = (Module *)vec_get(sources_g, source_i);
    proc_backend(&mod);
  }
}

static void proc_frontend(int source_i, DebugOption *debug_opt, Module **mod) {
  char *user_input = get_contents((*mod)->file_path);

  // step.1 tokenize
  Token *top_token = tokenize(user_input);
  debug_tokens_to_stderr(debug_opt->dbg_compiler, top_token);

  // step.2 parse
  Vector *functions = parse(top_token, source_i);
  dealloc_tokens(&top_token);

  // step.3 typecheck and allocating_stack
  for (int i = 0; i < functions->length; i++) {
    Function *iter_func = (Function *)vec_get(functions, i);
    debug_func_to_stderr(debug_opt->dbg_compiler, iter_func);
  }

  type_check(&functions);
  allocate_stack_frame(&functions);

  (*mod)->functions = functions;
}

static void proc_backend(Module **mod) {
  if ((*mod)->kind == MD_PRIMARY) {
    gen_x64_primary((*mod)->functions);
  } else {
    gen_x64_external(*mod);
  }

  if ((*mod)->strings->length > 0) {
    gen_x64_strlit(*mod);
  }

  for (int i = 0; i < (*mod)->functions->length; i++) {
    Function *iter_func = (Function *)vec_get((*mod)->functions, i);

    dealloc_function(iter_func);
  }
}
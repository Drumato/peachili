#include <sys/stat.h>

#include "bundler.h"
#include "module.h"
#include "token.h"
#include "util.h"
#include "vector.h"

#define FILE_SUFFIX_LENGTH 3

extern Token *tokenize(char *program);
static bool check_module_exists(char *path);
static char *try_get_module_path_from_envvar(char *path);
static bool already_visited_or_require_not_found(Module *mod, TokenKind kind);
static void parse_requires(Module **mod, Token **tok);
static void expect_symbol(Token **tok, char *pat);
static bool eat_if_symbol_matched(Token **tok, char *pat);
static char *expect_module_literal(Token **tok);
static char *copy_name_from_token(char **str);
static void check_module_name_is_valid(char **module_name);
static Token *prepare_for_next_module(Module **parent, char *module_name, Module **required_module);

void bundler_parse(Module **mod, Token **top_token) {
  vec_push(sources_g, *mod);

  // `require` はファイル先頭にしか存在しない．
  TokenKind cur_kind = (*top_token)->kind;
  if (already_visited_or_require_not_found(*mod, cur_kind)) {
    return;
  }

  // 相互参照時に無限ループしないようにフラグ設定
  (*mod)->visited = true;
  progress_token(top_token);

  parse_requires(mod, top_token);
}

static bool check_module_exists(char *path) {
  struct stat st;
  return stat(path, &st) == 0;
}

static char *try_get_module_path_from_envvar(char *path) {
  // 環境変数からのフルパスを構築
  char *full_path =
      str_alloc_and_copy(lib_path_env, strlen(lib_path_env) + strlen(path) + 1);
  int length = strlen(lib_path_env);

  // "/"があるかないか
  if (full_path[length] != '/') {
    strncpy(&(full_path[length]), "/", 1);
    length++;
  }

  strncpy(&(full_path[length]), path, strlen(path));
  full_path[length + strlen(path)] = '\0';

  if (check_module_exists(full_path)) {
    return full_path;
  }

  return NULL;
}

static bool already_visited_or_require_not_found(Module *mod, TokenKind kind) {
  return (mod->visited || kind != TK_REQUIRE);
}

static void parse_requires(Module **mod, Token **tok) {
  expect_symbol(tok, "(");

  while (!eat_if_symbol_matched(tok, ")")) {
    // モジュール名のパース，トークンからのコピー
    char *str = expect_module_literal(tok);
    char *required_module_name = copy_name_from_token(&str);

    // パースしたモジュール名の正当性をチェック．
    // ここではファイルが存在するかチェックする．
    // $PEACHILI_LIB_PATH以下に存在すればそのパスを格納しておく
    check_module_name_is_valid(&required_module_name);

    // 再帰的に呼び出す
    Module *required_module;
    Token *module_token = prepare_for_next_module(mod, required_module_name, &required_module);
    bundler_parse(&required_module, &module_token);

    dealloc_tokens(&module_token);

    eat_if_symbol_matched(tok, ",");
  }
}
static void expect_symbol(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL ||
      strncmp((*tok)->str, pat, strlen((*tok)->str))) {
    fprintf(stderr, "%d:%d: expected %s unexpected ", (*tok)->row, (*tok)->col,
            pat);
    dump_token(*tok);
    fprintf(stderr, "\n");
  }
  progress_token(tok);
}

// もし指定パターンにマッチすれば読みすすめる
static bool eat_if_symbol_matched(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL ||
      strncmp((*tok)->str, pat, strlen((*tok)->str)))
    return false;
  progress_token(tok);
  return true;
}

static char *expect_module_literal(Token **tok) {
  if ((*tok)->kind != TK_STRLIT) {
    fprintf(stderr, "module name must start with '\"'\n");
    exit(1);
  }
  char *ptr = (*tok)->str;
  progress_token(tok);
    
  return ptr;
}

static char *copy_name_from_token(char **str) {
  int length = strlen(*str);
  char *ptr = str_alloc_and_copy(*str, length);
  char *ext_ptr = ptr + length;
  strncpy(ext_ptr, ".go", FILE_SUFFIX_LENGTH);
  ext_ptr[FILE_SUFFIX_LENGTH] = '\0';

  return ptr;
}

static void check_module_name_is_valid(char **module_name) {
    // $PEACHILI_LIB_PATH か
    // 相対パスのどちらかに同名ファイルが存在しなければエラー．
    char *full_path = try_get_module_path_from_envvar(*module_name);
    if (check_module_exists(*module_name)) {
      return;
    }

    // $PEACHILI_LIB_PATH以下に存在しなかったとき
    if (full_path == NULL) {
      fprintf(stderr, "not found such a module -> %s\n",
              *module_name);
      exit(1);
    }

    *module_name = full_path;
}

static Token *prepare_for_next_module(Module **parent, char *module_name, Module **required_module) {
    char *module_input = get_contents(module_name);
    Token *module_token = tokenize(module_input);
    free(module_input);
    *required_module = new_module(MD_EXTERNAL, module_name);
    vec_push((*parent)->requires, (void *)*required_module);

    return module_token;
}

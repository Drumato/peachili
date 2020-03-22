#include <sys/stat.h>

#include "bundler.h"
#include "module.h"
#include "token.h"
#include "util.h"
#include "vector.h"

#define FILE_SUFFIX_LENGTH 3

extern Token *tokenize(char *program);
static bool check_module_exists(char *path);
static char *check_module_exists_from_envvar(char *path);

void bundler_parse(Module **mod, Token **top_token) {
  vec_push(sources_g, *mod);

  // `require` はファイル先頭にしか存在しない．
  if ((*mod)->visited || (*top_token)->kind != TK_REQUIRE) {
    return;
  }

  // 相互参照時に無限ループしないようにフラグ設定
  (*mod)->visited = true;
  *top_token = (*top_token)->next;

  // TODO: 今は()， つまり複数インポートには対応しない
  if ((*top_token)->kind != TK_STRLIT) {
    fprintf(stderr, "module name must start with '\"'\n");
    exit(1);
  }

  char *ptr = (*top_token)->str;
  *top_token = (*top_token)->next;

  char *required_module_name = str_alloc_and_copy(ptr, strlen(ptr));
  ptr = required_module_name + strlen(ptr);
  strncpy(ptr, ".go", FILE_SUFFIX_LENGTH);
  ptr[FILE_SUFFIX_LENGTH] = '\0';

  // $PEACHILI_LIB_PATH か
  // 相対パスのどちらかに同名ファイルが存在しなければエラー．
  char *full_path = check_module_exists_from_envvar(required_module_name);
  if (!check_module_exists(required_module_name)) {
    if (full_path == NULL) {
      fprintf(stderr, "not found such a module -> %s\n", required_module_name);
      exit(1);
    }
    required_module_name = full_path;
  }

  // 再帰的に呼び出す
  char *module_input = get_contents(required_module_name);
  Token *module_token = tokenize(module_input);
  Module *required_module = new_module(MD_EXTERNAL, required_module_name);
  vec_push((*mod)->requires, (void *)required_module);
  bundler_parse(&required_module, &module_token);

}

static bool check_module_exists(char *path) {
  struct stat st;
  return stat(path, &st) == 0;
}

static char *check_module_exists_from_envvar(char *path) {
  struct stat st;

  char *env_string = getenv("PEACHILI_LIB_PATH");
  assert(env_string);

  // 環境変数からのフルパスを構築
  char *full_path =
      str_alloc_and_copy(env_string, strlen(env_string) + strlen(path) + 1);

  int length = strlen(env_string);
  // "/"があるかないか
  if (full_path[length] != '/') {
    strncpy(&(full_path[length]), "/", 1);
    length++;
  }

  strncpy(&(full_path[length]), path, strlen(path));
  full_path[length + strlen(path)] = '\0';

  if (stat(full_path, &st) == 0) {
    return full_path;
  }

  return NULL;
}
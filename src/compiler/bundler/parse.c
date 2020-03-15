#include <sys/stat.h>

#include "bundler.h"
#include "module.h"
#include "token.h"
#include "vector.h"

#define FILE_SUFFIX_LENGTH 3

extern Token *tokenize(char *program);
extern char *get_contents(const char *filename);
static bool symbol_matched(Token **tok, char *pat);
static bool check_module_exists(char *path);

void bundler_parse(Module **mod, Token **top_token) {
  vec_push(sources_g, *mod);

  // `require` はファイル先頭にしか存在しない．
  if ((*mod)->visited || (*top_token)->kind != TK_REQUIRE) {
    (*mod)->requires = NULL;
    return;
  }

  // 相互参照時に無限ループしないようにフラグ設定
  (*mod)->visited = true;
  *top_token      = (*top_token)->next;

  // TODO: 今は()， つまり複数インポートには対応しない
  if (!symbol_matched(top_token, "\"")) {
    fprintf(stderr, "module name must start with '\"'\n");
    exit(1);
  }

  *top_token = (*top_token)->next;

  if ((*top_token)->kind != TK_IDENT) {
    fprintf(stderr, "module name must be an identifier -> %s\n", (*top_token)->str);
    exit(1);
  }

  char *ptr                  = (*top_token)->str;
  char *required_module_name = (char *)calloc(strlen(ptr) + FILE_SUFFIX_LENGTH, sizeof(char));
  strncpy(required_module_name, ptr, strlen(ptr));
  ptr = required_module_name + strlen(ptr);
  strncpy(ptr, ".go", FILE_SUFFIX_LENGTH);
  ptr[FILE_SUFFIX_LENGTH] = '\0';

  // $PEACHILI_STD_PATH か 相対パスのどちらかに同名ファイルが存在しなければエラー．
  if (!check_module_exists(required_module_name)) {
    fprintf(stderr, "not found such a module -> %s\n", required_module_name);
    exit(1);
  }

  // 再帰的に呼び出す
  char *module_input      = get_contents(required_module_name);
  Token *module_token     = tokenize(module_input);
  Module *required_module = new_module(MD_EXTERNAL, required_module_name);
  vec_push((*mod)->requires, (void *)required_module);
  bundler_parse(&required_module, &module_token);

  free(required_module_name);
}

static bool check_module_exists(char *path) {
  // TODO: 環境変数には今の所対応させない．
  struct stat st;
  if (stat(path, &st) == 0) {
    return true;
  }
  return false;
}
static bool symbol_matched(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL || strncmp((*tok)->str, pat, strlen((*tok)->str))) return false;
  return true;
}

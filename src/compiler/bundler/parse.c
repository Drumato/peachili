#include <dirent.h>
#include <sys/stat.h>

#include "bundler.h"
#include "module.h"
#include "token.h"
#include "util.h"
#include "vector.h"

#define FILE_SUFFIX_LENGTH 3

extern Token *tokenize(char *program);

// main procedures
static void parse_requires(Module **mod, Token **tok);
static char *resolve_file_path(char **module_name);
static Token *prepare_for_next_module(char *module_name, int s_len,
                                      Module **required_module);
static void process_each_module_name(Module **mod, Token **tok);

static char *try_get_module_path_from_envvar(char *path);

// utilities
static bool check_file_path_exists(char *path);
static bool already_visited_or_require_not_found(Module *mod, TokenKind kind);
static void expect_symbol(Token **tok, char *pat);
static bool eat_if_symbol_matched(Token **tok, char *pat);
static char *expect_module_literal(Token **tok);
static char *copy_name_from_token(char **str);
static char *append_extension(char **str);
static char *concat_dir_and_sub(char **dir, char **sub);
static bool check_dot_directory(char *dir_name);
static char *fg_cur_dir = NULL;

void bundler_parse(Module **mod, Token **top_token) {

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

static void parse_requires(Module **mod, Token **tok) {
  expect_symbol(tok, "(");

  while (!eat_if_symbol_matched(tok, ")")) {
    process_each_module_name(mod, tok);
  }
}

static void process_each_module_name(Module **mod, Token **tok) {
  // モジュール名のパース，トークンからのコピー
  char *str = expect_module_literal(tok);
  char *mod_name = copy_name_from_token(&str);
  char *required_module_name = append_extension(&mod_name);

  char *resolved;

  // ディレクトリインポートの場合
  {
    if ((resolved = resolve_file_path(&mod_name)) != NULL) {
      fg_cur_dir = mod_name;
      Module *dir_module = new_module(MD_EXTERNAL, resolved);
      vec_push((*mod)->requires, (void *)dir_module);
      vec_push(sources_g, (void *)dir_module);

      DIR *dir;
      struct dirent *dp;

      if ((dir = opendir(resolved)) == NULL) {
        fprintf(stderr, "opendir(%s) failed\n", resolved);
        exit(1);
      }

      while ((dp = readdir(dir)) != NULL) {
        if (check_dot_directory(dp->d_name)) {
          continue;
        }

        Module *in_dir_mod;

        int length = strlen(dp->d_name);
        char *sub_mod = (char *)calloc(length, sizeof(char));
        strncpy(sub_mod, dp->d_name, length);
        sub_mod[length] = '\0';

        char *dir_path = concat_dir_and_sub(&resolved, &sub_mod);

        int s_len = strlen(dir_path);
        Token *dir_token =
            prepare_for_next_module(dir_path, s_len, &in_dir_mod);
        vec_push(dir_module->subs, (void *)in_dir_mod);
        bundler_parse(&in_dir_mod, &dir_token);

        dealloc_tokens(&dir_token);
      }

      if (dir != NULL) {
        closedir(dir);
      }

      fg_cur_dir = NULL;
      eat_if_symbol_matched(tok, ",");
      return;
    }
  }

  // ファイルインポートの場合
  {

    // ここでNULLの場合，ディレクトリもファイルも見つからなかった，ということになる
    // つまりエラーを吐いて終了すればいい．
    if (fg_cur_dir != NULL) {
      required_module_name =
          concat_dir_and_sub(&fg_cur_dir, &required_module_name);
    }

    if ((resolved = resolve_file_path(&required_module_name)) == NULL) {
      fprintf(stderr, "resolve failed\ninvalid module name -> %s\n",
              required_module_name);
      exit(1);
    }

    // 再帰的に呼び出す
    Module *required_module;
    int length = strlen(resolved);
    Token *module_token =
        prepare_for_next_module(resolved, length, &required_module);
    vec_push((*mod)->requires, (void *)required_module);
    bundler_parse(&required_module, &module_token);
    vec_push(sources_g, (void *)required_module);

    dealloc_tokens(&module_token);

    eat_if_symbol_matched(tok, ",");
  }
}

static char *resolve_file_path(char **module_name) {
  // 相対パスで検索できるか
  if (check_file_path_exists(*module_name)) {
    return *module_name;
  }

  // $PEACHILI_LIB_PATH 以下に存在するかチェック
  char *full_path = try_get_module_path_from_envvar(*module_name);

  if (full_path != NULL) {
    return full_path;
  }

  return NULL;
}

static char *try_get_module_path_from_envvar(char *path) {
  // 環境変数からのフルパスを構築
  int length = strlen(lib_path_env);
  char *full_path = (char *)calloc(length + strlen(path) + 2, sizeof(char));
  strncpy(full_path, lib_path_env, length);

  // "/"があるかないか
  if (full_path[length - 1] != '/') {
    strncpy(&(full_path[length]), "/", 1);
    length++;
  }

  // 環境変数の文字列とモジュール名の連結
  strncpy(&(full_path[length]), path, strlen(path));
  full_path[length + strlen(path)] = '\0';

  if (check_file_path_exists(full_path)) {
    return full_path;
  }

  return NULL;
}

static Token *prepare_for_next_module(char *module_name, int s_len,
                                      Module **required_module) {

  char *file_path = (char *)calloc(s_len + 1, sizeof(char));
  strncpy(file_path, module_name, s_len);
  file_path[s_len] = 0x00;

  char *module_input = get_contents(file_path);
  Token *module_token = tokenize(module_input);
  free(module_input);
  *required_module = new_module(MD_EXTERNAL, file_path);

  return module_token;
}

static bool already_visited_or_require_not_found(Module *mod, TokenKind kind) {
  return (mod->visited || kind != TK_REQUIRE);
}

static bool check_file_path_exists(char *path) {
  struct stat st;
  return stat(path, &st) == 0;
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
  char *ptr = (char *)calloc(length + 1, sizeof(char));
  strncpy(ptr, *str, length);
  ptr[length] = 0;
  return ptr;
}

static char *append_extension(char **str) {
  int length = strlen(*str);
  char *ptr = (char *)calloc(length + FILE_SUFFIX_LENGTH + 1, sizeof(char));
  strncpy(ptr, *str, length);

  // freeしたくなるが，すべきではない．
  // 拡張子を連結する前の文字列も使用するからである．
  // free(*str);

  strncpy(ptr + length, ".go", FILE_SUFFIX_LENGTH);
  ptr[length + FILE_SUFFIX_LENGTH] = '\0';

  return ptr;
}

static char *concat_dir_and_sub(char **dir, char **sub) {
  int dir_len = strlen(*dir);
  int sub_len = strlen(*sub);
  char *ptr = (char *)calloc(dir_len + sub_len + 2, sizeof(char));
  strncpy(ptr, *dir, dir_len);

  // "/"があるかないか
  strncpy(&(ptr[dir_len]), "/", 1);
  dir_len++;

  // 環境変数の文字列とモジュール名の連結
  strncpy(&(ptr[dir_len]), *sub, sub_len);
  ptr[dir_len + sub_len] = '\0';

  if (!check_file_path_exists(ptr)) {
    fprintf(stderr, "not found -> %s\n", ptr);
    exit(1);
  }

  return ptr;
}

static bool check_dot_directory(char *dir_name) {
  bool one_dot = !strncmp(dir_name, ".", 1) && (strlen(dir_name) == 1);
  bool two_dot = !strncmp(dir_name, "..", 2) && (strlen(dir_name) == 2);

  return one_dot || two_dot;
}
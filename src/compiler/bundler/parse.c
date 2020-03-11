#include "module.h"
#include "token.h"

extern Token *tokenize(char *program);
static bool symbol_matched(Token **tok, char *pat);

void bundler_parse(Module **mod, Token **top_token) {
  if ((*top_token)->kind != TK_REQUIRE) {
    (*mod)->requires = NULL;
    return;
  }

  (*mod)->visited = true;
  *top_token      = (*top_token)->next;

  // TODO: 今は()， つまり複数インポートには対応しない
  if (!symbol_matched(top_token, "\"")) {
    fprintf(stderr, "module name must start with '\"'\n");
    exit(1);
  }

  *top_token = (*top_token)->next;

  if ((*top_token)->kind != TK_IDENT) {
    fprintf(stderr, "invalid module name -> %s\n", (*top_token)->str);
    exit(1);
  }

  char *required_module_name = (*top_token)->str;
  fprintf(stderr, "required module found -> %s\n", required_module_name);
}

static bool symbol_matched(Token **tok, char *pat) {
  if ((*tok)->kind != TK_SYMBOL || strncmp((*tok)->str, pat, strlen((*tok)->str))) return false;
  return true;
}

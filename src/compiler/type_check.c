#include "agtype.h"
#include "ast.h"
#include "variable.h"
#include "vector.h"

static void type_check_in_func(Function **func);

static Vector *fg_vec;
void type_check(Vector **functions) {
  fg_vec = *functions;

  for (int i = 0; i < (*functions)->length; i++) {
    Function *iter_func = (Function *)vec_get(*functions, i);

    if (iter_func->kind == FN_DEFINED) {
      type_check_in_func(&iter_func);
    }
  }
}

void type_check_in_func(Function **func) {
  for (int i = 0; i < (*func)->stmts->length; i++) {
    Node *n = get_statement(*func, i);
    switch (n->kind) {
      case ND_COUNTUP:
        if (n->expr->kind != ND_IDENT) {
          fprintf(stderr, "%d:%d: countup-expr must be an identifier in countup-statement\n",
                  n->row, n->col);
          exit(1);
        }
        break;
      case ND_ASSIGN:
        if (n->left->kind != ND_IDENT) {
          fprintf(stderr, "%d:%d: left-value must be an identifier in assignment\n", n->row,
                  n->col);
          exit(1);
        }
        break;
      default:
        break;
    }
  }
}

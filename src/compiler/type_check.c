#include "agtype.h"
#include "ast.h"
#include "variable.h"
#include "vector.h"
void type_check(Function **func) {
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

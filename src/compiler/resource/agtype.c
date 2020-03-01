#include "agtype.h"

AGType *new_integer_type(void) {
  AGType *agtype = (AGType *)calloc(1, sizeof(AGType));
  agtype->kind   = TY_INT;
  agtype->size   = 8;
  return agtype;
}

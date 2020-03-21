#include "agtype.h"

AGType *new_integer_type(void) {
  AGType *agtype = (AGType *)calloc(1, sizeof(AGType));
  agtype->kind = TY_INT;
  agtype->size = 8;
  return agtype;
}

void dump_agtype(AGType *agtype) {
  switch (agtype->kind) {
  case TY_INT:
    fprintf(stderr, "int");
  }
}

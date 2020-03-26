#include "agtype.h"

AGType *new_integer_type(void) {
  AGType *agtype = (AGType *)calloc(1, sizeof(AGType));
  agtype->kind = TY_INT;
  agtype->size = 8;
  return agtype;
}

AGType *new_string_type(void) {
  AGType *agtype = (AGType *)calloc(1, sizeof(AGType));
  agtype->kind = TY_STR;
  agtype->size = 8; // pointer size
  return agtype;
}

AGType *new_noreturn_type(void) {
  AGType *agtype = (AGType *)calloc(1, sizeof(AGType));
  agtype->kind = TY_NORETURN;
  agtype->size = 0;
  return agtype;
}

void dump_agtype(AGType *agtype) {
  switch (agtype->kind) {
  case TY_INT:
    fprintf(stderr, "int");
    break;
  case TY_STR:
    fprintf(stderr, "str");
    break;
  case TY_NORETURN:
    fprintf(stderr, "noreturn");
    break;
  }
}

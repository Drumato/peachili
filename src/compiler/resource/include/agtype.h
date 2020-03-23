#include "base.h"

typedef enum {
  TY_INT,
  TY_NORETURN,
} AGTypeKind;

struct AGType {
  AGTypeKind kind;
  uint32_t size;
};
typedef struct AGType AGType;

AGType *new_integer_type(void);
AGType *new_noreturn_type(void);
void dump_agtype(AGType *agtype);

#include "base.h"

typedef enum {
  TY_INT,
} AGTypeKind;

struct AGType {
  AGTypeKind kind;
  uint32_t size;
};
typedef struct AGType AGType;

AGType *new_integer_type(void);
void dump_agtype(AGType *agtype);

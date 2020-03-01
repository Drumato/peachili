#include "base.h"

typedef enum {
  TY_INT,
} AGTypeKind;

typedef struct {
  AGTypeKind kind;
  uint32_t size;
} AGType;

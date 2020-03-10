#include "agtype.h"
#include "ast.h"
#include "variable.h"
#include "vector.h"

void allocate_stack_frame(Function **func) {
  int total_offset = 0;
  for (int i = 0; i < (*func)->locals->length; i++) {
    Variable *var = get_local_var(*func, i);

    // 型のサイズを取得し，累積のオフセットを割り当てる
    total_offset += var->type->size;
    var->offset = total_offset;
  }
  (*func)->stack_offset = total_offset;
}

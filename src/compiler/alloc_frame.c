#include "agtype.h"
#include "ast.h"
#include "variable.h"
#include "vector.h"

static void alloc_frame_in_func(Function **func);

void allocate_stack_frame(Vector **functions) {

  for (int i = 0; i < (*functions)->length; i++) {
    Function *iter_func = (Function *)vec_get(*functions, i);

    alloc_frame_in_func(&iter_func);
  }
}

static void alloc_frame_in_func(Function **func) {
  int total_offset = 0;
  for (int i = 0; i < (*func)->locals->length; i++) {
    Variable *var = get_local_var(*func, i);

    // 型のサイズを取得し，累積のオフセットを割り当てる
    total_offset += var->type->size;
    var->offset = total_offset;
  }
  (*func)->stack_offset = total_offset;
}

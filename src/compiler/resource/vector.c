#include "vector.h"

#include <assert.h>
#include <stdlib.h>

Vector *new_vec(void) {
  Vector *vec = malloc(sizeof(Vector));
  vec->data = malloc(sizeof(void *) * 16);
  vec->capacity = 16;
  vec->length = 0;
  return vec;
};

void *vec_get(Vector *vec, int idx) {
  assert(idx >= 0 && idx < vec->length);
  return vec->data[idx];
}

void vec_resize(Vector *vec) {
  vec->capacity *= 2;
  vec->data = realloc(vec->data, sizeof(void *) * vec->capacity);
}

void vec_push(Vector *vector, void *elem) {
  if (vector->length >= vector->capacity)
    vec_resize(vector);
  vector->data[vector->length++] = elem;
}

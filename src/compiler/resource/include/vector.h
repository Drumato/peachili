struct Vector {
  int length;   // elements of number
  int capacity; // size of array(allocated)
  void **data;
};
typedef struct Vector Vector;

Vector *new_vec(void);
void *vec_get(Vector *vec, int idx);
void vec_resize(Vector *vec);
void vec_push(Vector *vector, void *elem);

#include "base.h"

FILE *open_file(const char *filename, const char *mode) {
  FILE *file;
  file = fopen(filename, mode);
  if (file == NULL) {
    fprintf(stderr, "Error Found: %s -> '%s'\n", strerror(errno), filename);
    exit(1);
  }
  return file;
}

struct stat get_file_info(FILE *file) {
  struct stat st;
  int fd = fileno(file);
  if (fstat(fd, &st) != 0) {
    fprintf(stderr, "unable to get file information\n");
    fprintf(stderr, "%s\n", strerror(errno));
    exit(1);
  }
  return st;
}

char *get_contents(const char *filename) {
  FILE *file = open_file(filename, "rb");
  struct stat st = get_file_info(file);
  size_t nmemb = (size_t)(st.st_size);
  char *buf = (char *)malloc(nmemb);
  if (fread(buf, (size_t)(1), nmemb, file) < nmemb) {
    fprintf(stderr, "Error Found:%s -> %s\n", strerror(errno), filename);
    exit(1);
  }
  fclose(file);
  buf[nmemb] = '\0';
  return buf;
}

int aligned_strlen(char *ptr) {
  int length = strlen(ptr);
  int aligned = length + 15;
  aligned &= ~15;
  return aligned;
}
char *str_alloc_and_copy(char *src, int length) {

  char *allocated = (char *)calloc(length, sizeof(char));
  strncpy(allocated, src, length);
  allocated[length] = 0;
  return allocated;
}

char *get_last_path(char *filename) {
  if (strchr(filename, '/') == NULL) {
    return filename;
  }
  char *fp = filename;
  char *prev;
  char *next = strtok(fp, "/");
  while (next != NULL) {
    prev = next;
    next = strtok(NULL, "/");
  }
  return prev;
}
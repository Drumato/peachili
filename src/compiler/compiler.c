#include "base.h"
#include "structure.h"

void compiler_main(int argc, char **argv, DebugOption *debug_opt) {
  if (argc < 2) {
    fprintf(stderr, "invalid arguments.\n usage: %s <source-file>\n", argv[0]);
  }

  if (debug_opt->verbose) {
    fprintf(stderr, "start compiling with verbosity...\n");
  }
}

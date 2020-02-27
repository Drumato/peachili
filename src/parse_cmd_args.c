#include "structure.h"
void parse_arguments(int argc, char **argv,
                     DebugOption *debug_opt) {
  struct option longopts[] = {
      {"verbose", no_argument, NULL, 'v'},
      {0, 0, 0, 0},
  };
  int opt, longindex;
  while ((opt = getopt_long(argc, argv, "v:::", longopts,
                            &longindex)) != -1) {
    switch (opt) {
      case 'v':
        debug_opt->verbose = true;
      default:
        break;
    }
  }
}


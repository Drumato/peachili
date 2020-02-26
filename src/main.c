#include "structure.h"
extern void compiler_main(int argc, char **argv, DebugOption *debug_opt);
extern void parse_arguments(int argc, char **argv, DebugOption *debug_opt);

int main(int argc, char **argv) { 
  DebugOption debug_opt;

  parse_arguments(argc, argv, &debug_opt);

  compiler_main(argc, argv, &debug_opt); 
}

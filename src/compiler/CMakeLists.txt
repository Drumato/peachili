add_library(compiler
  SHARED
    agc.c
    lex.c
    parse.c
    alloc_frame.c
    type_check.c
    codegen.c
    resource/token.c
    resource/ast.c
    resource/agtype.c
    resource/vector.c
    resource/variable.c
)

target_include_directories(
  compiler
  PUBLIC ${PROJECT_SOURCE_DIR}/src/compiler/resource/include
  PUBLIC ${PROJECT_SOURCE_DIR}/src/include
)

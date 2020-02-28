add_library(compiler
  SHARED
    agc.c
    lex.c
    parse.c
    codegen.c
)

add_subdirectory(resource)

target_include_directories(
  compiler
  PUBLIC ${PROJECT_SOURCE_DIR}/src/compiler/resource/include
  PUBLIC ${PROJECT_SOURCE_DIR}/src/include
)

file(GLOB COMPILER_SOURCE *.c resource/*.c)
add_library(compiler
  SHARED
  ${COMPILER_SOURCE}
)

target_include_directories(
  compiler
  PUBLIC ${PROJECT_SOURCE_DIR}/src/compiler/resource/include
  PUBLIC ${PROJECT_SOURCE_DIR}/src/include
)

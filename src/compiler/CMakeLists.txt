file(GLOB COMPILER_SOURCE *.c resource/*.c bundler/*.c)
add_library(compiler
  SHARED
  ${COMPILER_SOURCE}
)

target_include_directories(
  compiler
  PUBLIC ${PROJECT_SOURCE_DIR}/src/compiler/bundler/include
  PUBLIC ${PROJECT_SOURCE_DIR}/src/compiler/resource/include
  PUBLIC ${PROJECT_SOURCE_DIR}/src/compiler/include
  PUBLIC ${PROJECT_SOURCE_DIR}/src/include
)

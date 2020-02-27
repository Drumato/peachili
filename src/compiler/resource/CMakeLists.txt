add_library(resource
  SHARED
    token.c
    ast.c
)

target_include_directories(
  resource
  PUBLIC ${PROJECT_SOURCE_DIR}/src/compiler/resource/include
  PUBLIC ${PROJECT_SOURCE_DIR}/src/include
  )

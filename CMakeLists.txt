cmake_minimum_required(VERSION 3.13.4)
project(peachili C)
set(CMAKE_C_FLAGS "-Wall -Wextra -std=c11")
set(CMAKE_C_FLAGS_DEBUG "-g")
set(CMAKE_C_COMPILER "/usr/bin/clang")

# サブディレクトリを登録
add_subdirectory(src)

file(GLOB TOP_SOURCE src/*.c)
add_executable(peachili ${TOP_SOURCE})

target_link_libraries(peachili compiler)
target_include_directories(peachili 
  PUBLIC ${PROJECT_SOURCE_DIR}/src/include,
  PUBLIC ${PROJECT_SOURCE_DIR}/src/compiler/resource/include
  )

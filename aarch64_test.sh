#!/bin/bash
build_peachili_executable() {
  input="$1"
  ../../target/debug/peachili compile "$input" --target aarch64
  rustc_actual="$?"
  if [ $rustc_actual -ne 0 ]; then
    echo -e "\e[31mbuilding an executable binary failed!\e[m"
    exit 1
  fi
}

try() {
  expected="$1"
  input="$2"

  # テストファイルのコンパイル
  build_peachili_executable $input

  clang-10 asm.s --target=aarch64-linux-gnu -static
  ./a.out
  actual="$?"
  rm a.out

  if [ "$actual" = "$expected" ]; then
    echo -e "$input => \e[32m$actual\e[m"
  else
    echo -e "$input => \e[32m$expected\e[m expected, but got \e[31m$actual\e[m"
    exit 1
  fi
}

echo -e "start to test normal program...\n\n"

cd examples/aarch64

try 42 "intlit.go"
try 9 "four_arith.go"

echo -e "\n\nOK"

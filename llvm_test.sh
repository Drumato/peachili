#!/bin/bash
build_peachili_executable() {
  input="$1"
  ../target/debug/peachili "$input" --target=llvm-ir
  clang ir.ll -o tmp
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

  ./tmp
  actual="$?"
  rm tmp*

  if [ "$actual" = "$expected" ]; then
    echo -e "$input => \e[32m$actual\e[m"
  else
    echo -e "$input => \e[32m$expected\e[m expected, but got \e[31m$actual\e[m"
    exit 1
  fi
}

echo -e "start to test normal program...\n\n"

cd examples

try 0 "empty_main.go"
try 0 "intlit.go"
try 9 "four_arith.go"
try 9 "unary_minus.go"
try 0 "unary_plus.go"
try 30 "declare_autovar.go"
try 9 "declare_twovar.go"
try 9 "countup.go"
try 30 "with_argument.go"
try 30 "without_argument.go"
try 3 "exit.go"
try 0 "hello_world.go"
try 15 "boolean_1.go"
try 30 "boolean_2.go"
try 30 "type_alias.go"
try 1 "unsigned_int.go"
try 30 "varinit.go"

echo -e "\n\nOK"
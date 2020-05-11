#!/bin/bash
build_and_test_peadchili_executable() {
  input="$1"
  ../target/debug/peachili "$input" -L
  rustc_actual="$?"
  if [ $rustc_actual -ne 1 ]; then
    echo -e "\e[31mcompiler must detect with one or more errors, but not.\e[m"
    exit 1
  fi
}

echo -e "start to test invalid program...\n\n"

cd failures

build_and_test_peadchili_executable "invalid_integer_literal.go"

echo -e "\n\nOK"
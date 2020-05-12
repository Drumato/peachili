#!/bin/bash
build_and_test_peadchili_executable() {
  input="$1"
  ../target/debug/peachili "$input" -L
  rustc_actual="$?"
  if [ $rustc_actual -ne 1 ]; then

    echo -e "$input => \e[31mcompiler must detect with one or more errors, but not.\e[m"
    exit 1
  else
    echo -e "$input => \e[32mpassed\e[m"
  fi
}

echo -e "start to test invalid program...\n\n"

cd failures

build_and_test_peadchili_executable "invalid_integer_literal.go"
build_and_test_peadchili_executable "if_expr1.go"
build_and_test_peadchili_executable "if_expr2.go"
build_and_test_peadchili_executable "if_expr_stmt1.go"
build_and_test_peadchili_executable "if_expr_stmt2.go"
build_and_test_peadchili_executable "invalid_assignment.go"

echo -e "\n\nOK"
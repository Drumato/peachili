#!/bin/bash
build_and_test_peachili_executable() {
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

build_and_test_peachili_executable "invalid_integer_literal.go"
build_and_test_peachili_executable "if_expr1.go"
build_and_test_peachili_executable "if_expr2.go"
build_and_test_peachili_executable "if_expr_stmt1.go"
build_and_test_peachili_executable "if_expr_stmt2.go"
build_and_test_peachili_executable "invalid_assignment.go"
build_and_test_peachili_executable "add_int64_and_boolean.go"
build_and_test_peachili_executable "uint_and_sint.go"
build_and_test_peachili_executable "assignment_to_constant.go"
build_and_test_peachili_executable "invalid_arg_types.go"
build_and_test_peachili_executable "invalid_arg_number.go"
build_and_test_peachili_executable "minus_to_unsigned.go"
build_and_test_peachili_executable "return_in_noreturn_func.go"

echo -e "\n\nOK"
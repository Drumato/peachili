#!/bin/bash
try() {
  expected="$1"
  input="$2"

  ../build/peachili "$input" > tmp.s
  gcc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

echo -e "start to test normal program...\n\n"

cd examples

try 0 "intlit.go"
try 9 "four_arith.go"
try 9 "unary_minus.go"
try 0 "unary_plus.go"
try 30 "declare_autovar.go"
try 9 "declare_twovar.go"
try 1 "if_expr_stmt1.go"
try 0 "if_expr_stmt2.go"
try 1 "if_expr1.go"
try 0 "if_expr2.go"
try 9 "countup.go"
try 30 "with_argument.go"
try 30 "without_argument.go"
try 30 "import.go"

echo -e "\n\nOK"

error() {
  input="$1"

  ./build/peachili "$input"
  actual="$?"

  if [ "$actual" = 1 ]; then
    echo "$input => $actual"
  else
    echo "should invoke an error in $input, but not worked."
    exit 1
  fi
}

rm tmp*

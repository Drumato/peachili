#!/bin/bash
try() {
  expected="$1"
  input="$2"

  ./build/peachili "$input" > tmp.s
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

try 0 "examples/intlit.go"
try 9 "examples/four_arith.go"
try 9 "examples/unary_minus.go"
try 0 "examples/unary_plus.go"
try 30 "examples/declare_autovar.go"
try 9 "examples/declare_twovar.go"
try 1 "examples/if_expr_stmt1.go"
try 0 "examples/if_expr_stmt2.go"
try 1 "examples/if_expr1.go"
try 0 "examples/if_expr2.go"
try 9 "examples/countup.go"

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

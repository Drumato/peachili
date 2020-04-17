#!/bin/bash
try() {
  expected="$1"
  input="$2"

<<<<<<< HEAD
  ../build/peachili "$input" > tmp.s
  gcc -static -o tmp tmp.s
=======
  ../target/debug/peachili "$input" -S
  gcc -static -o tmp asm.s
>>>>>>> codegen...!
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
try 3 "exit.go"
try 1 "use_version.go"
try 0 "hello_world.go"

echo -e "\n\nOK"

<<<<<<< HEAD
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
=======
rm tmp* asm.s
>>>>>>> codegen...!

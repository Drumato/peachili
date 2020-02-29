#!/bin/bash
try() {
  expected="$1"
  input="$2"

  ./build/dagc "$input" > tmp.s
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

try 0 "return 0;"
try 42 "return 42;"
try 21 "return 5 + 20 - 4;"
try 1 "return 1 * 10 / 10;"
try 9 "return 1 + 2 * 4;"
try 9 "return 0 + -3 * -3;"
try 0 "return 0 + -3 + +3;"

echo "OK"
rm tmp*

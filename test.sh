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

try 0 "func main() int { return 0;  }"
try 42 "func main() int { return 42; }"
try 21 "func main() int { return 5 + 20 - 4; }"
try 1 "func main() int { return 1 * 10 / 10; }"
try 9 "func main() int { return 1 + 2 * 4; }"
try 9 "func main() int { return 0 + -3 * -3; }"
try 0 "func main() int { return 0 + -3 + +3; }"
try 30 "func main() int { declare x int; x = 30; return x ; }"
try 3 "func main() int { declare x int; declare y int; x = 1; y = 3; return x * y; }"
try 9 "func main() int { declare x int; declare y int; x = y = 3; return x * y; }"
try 1 "func main() int { if(1){ return 1; }; return 0;}"
try 0 "func main() int { if(0){ return 1; }; return 0;}"
try 1 "func main() int { declare x int; x = if(1){ ifret 1; } else { ifret 0; }; return x;}"
try 0 "func main() int { declare x int; x = if(0){ ifret 1; } else { ifret 0; }; return x;}"
try 9 "func main() int { declare res int; countup x int from 0 to 10 { res = x; }; return res; }"

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

echo -e "start to test invalid program...\n\n"
error "func main() int { declare x int; 3 = 30; return x; }"

echo -e "\n\nOK"
rm tmp*

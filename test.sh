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

try 0 "func main() int { return 0;  }"
try 42 "func main() int { return 42; }"
try 21 "func main() int { return 5 + 20 - 4; }"
try 1 "func main() int { return 1 * 10 / 10; }"
try 9 "func main() int { return 1 + 2 * 4; }"
try 9 "func main() int { return 0 + -3 * -3; }"
try 0 "func main() int { return 0 + -3 + +3; }"
try 30 "func main() int { var x int; x = 30; return x ; }"
try 3 "func main() int { var x int; var y int; x = 1; y = 3; return x * y; }"
try 9 "func main() int { var x int; var y int; x = y = 3; return x * y; }"
try 1 "func main() int { if(1){ return 1; }; return 0;}"
try 0 "func main() int { if(0){ return 1; }; return 0;}"

echo "OK"
rm tmp*

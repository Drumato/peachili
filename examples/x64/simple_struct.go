import x64;

struct A {
    foo Int64
    bar Int64
}

func main() Noreturn {
  declare a A;
  a.foo = 15;
  a.bar = 30;

  varinit res Int64 = a.foo + a.bar;
	x64::exit_with(res);
}

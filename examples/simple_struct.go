require (
    "std"
)

struct A {
    foo Int64
    bar Int64
}

func main() Noreturn {
    declare a A;
    a.foo = 15;
    a.bar = 30;

    varinit res Int64 = a.foo + a.bar;
	std::os::exit_with(res);
}

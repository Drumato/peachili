require (
    "std"
)

struct X {
    exist Boolean
}

func main() Noreturn {
    declare x X;
	x.foo = true;
	std::os::exit_with(x.foo);
}

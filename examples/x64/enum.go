import x64;

pubenum A {
    B,
    C,
}

func main() Noreturn {
    varinit x A = A::B;

    match x {
        A::B -> {
            x64::exit_with(2);
        },
        A::C -> {
            x64::exit_with(4);
        },
    };
}

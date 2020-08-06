import x64;

func foo(x Int64, y Int64) Int64 {
	return x + y; 
}

func main() Noreturn {
    x64::exit_with(foo(10, 20));
}

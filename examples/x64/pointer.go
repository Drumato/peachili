import x64;

func main() Noreturn {
    varinit x Int64 = 4;
    varinit y *Int64 = &x;
		x64::exit_with(*y);
}

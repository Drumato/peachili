import x86_64;

func main() Noreturn {
	declare x Int64;
	x = 30;
	x86_64::exit_with(x);
}
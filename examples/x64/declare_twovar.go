import x64;

func main() Noreturn {
	declare x Int64;
	declare y Int64;
	x = y = 3;
	x64::exit_with(x * y);
}

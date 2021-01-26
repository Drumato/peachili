import x86_64;

func main() Noreturn {
	declare x Int64;
	declare y Int64;
	x = 30;
	y = 10;
	x = 5;
	x86_64::exit_with(x + y);
}

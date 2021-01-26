import aarch64;

func main() Noreturn {
	declare x Int64;
	declare y Int64;
	x = 30;
	y = 10;
	x = 5;
	aarch64::exit_with(x + y);
}

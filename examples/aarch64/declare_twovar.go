import aarch64;

func main() Noreturn {
	declare x Int64;
	declare y Int64;
	x = 3;
    y = 3;
	aarch64::exit_with(x * y);
}

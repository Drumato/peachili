import aarch64;

func main() Noreturn {
	declare x Int64;
	x = 30;
	aarch64::exit_with(x);
}

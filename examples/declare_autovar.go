require "os";

func main() noreturn {
	declare x int;
	x = 30;
	os::exit_with(x);
}

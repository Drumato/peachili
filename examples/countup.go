require "os";

func main() noreturn {
	declare res int; 
	countup x int from 0 to 10 { 
		res = x; 
	};
	os::exit_with(res);
}

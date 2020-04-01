require (
    "std"
)

func main() noreturn {
	declare res int; 
	countup x int from 0 to 10 { 
		res = x; 
	};
	std::os::exit_with(res);
}

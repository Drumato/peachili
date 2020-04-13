require (
    "std"
)

func main() noreturn {
	declare res int64;
	countup x int64 from 0 to 10 {
		res = x; 
	};
	std::os::exit_with(res);
}

require (
    "std"
)

func main() noreturn {
	declare x int;
	x = 30;
	std::os::exit_with(x);
}

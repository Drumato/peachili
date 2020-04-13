require (
    "std"
)

func main() noreturn {
	declare x int64;
	x = 30;
	std::os::exit_with(x);
}

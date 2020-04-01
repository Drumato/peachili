require (
    "std"
)
func main() noreturn {
	declare x int;
	declare y int;
	x = y = 3;
	std::os::exit_with(x * y);
}

require (
    "std"
)
func main() noreturn {
	declare x int64;
	declare y int64;
	x = y = 3;
	std::os::exit_with(x * y);
}

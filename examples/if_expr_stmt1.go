require (
    "std"
)
func main() noreturn {
	if (1) {
		std::os::exit_with(1);
	};
	std::os::exit_with(0);
}

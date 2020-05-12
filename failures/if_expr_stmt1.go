require (
    "std"
)
func main() noreturn {

    // if式内の条件式はboolean型のみ
	if (1) {
		std::os::exit_with(1);
	};
	std::os::exit_with(0);
}

require (
    "std"
)
func main() Noreturn {

    // if式内の条件式はboolean型のみ
	if (0) {
	    std::os::exit_with(1);
	};
	std::os::exit_with(0);
}

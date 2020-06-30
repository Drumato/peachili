require (
    "std"
)
func main() Noreturn {

    // if式内の条件式はBoolean型のみ
	if (1) {
		std::os::exit_with(1);
	};
	std::os::exit_with(0);
}

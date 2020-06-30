require (
    "std"
)
func main() Noreturn {
	declare x Int64;

    // if式内の条件式はBoolean型のみ
	x = if (1) {
		ifret 1;
	} else {
		ifret 0;
	};

	std::os::exit_with(x);
}

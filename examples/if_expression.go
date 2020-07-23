import std;

func main() Noreturn {
	declare x Int64;
	x = if(true) {
		ifret 1;
	} else {
		ifret 2;
	};
	std::os::exit_with(x);
}

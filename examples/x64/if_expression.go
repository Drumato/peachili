import x64;

func main() Noreturn {
	declare x Int64;
	x = if(true) {
		ifret 1;
	} else {
		ifret 2;
	};
	x64::exit_with(x);
}

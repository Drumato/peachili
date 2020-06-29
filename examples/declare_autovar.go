require (
    "std"
)

func main() Noreturn {
	declare x Int64;
	x = 30;
	std::os::exit_with(x);
}

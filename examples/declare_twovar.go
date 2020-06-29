require (
    "std"
)
func main() Noreturn {
	declare x Int64;
	declare y Int64;
	x = y = 3;
	std::os::exit_with(x * y);
}

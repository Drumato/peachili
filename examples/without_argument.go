require (
    "std"
)
func foo() int {
	return 30; 
}
func main() noreturn {
	std::os::exit_with(foo());
}

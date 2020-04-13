require (
    "std"
)
func foo() int64 {
	return 30; 
}
func main() noreturn {
	std::os::exit_with(foo());
}

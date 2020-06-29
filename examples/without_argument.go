require (
    "std"
)
func foo() Int64 {
	return 30; 
}
func main() Noreturn {
	std::os::exit_with(foo());
}

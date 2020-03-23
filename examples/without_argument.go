require "os";
func foo() int {
	return 30; 
}
func main() noreturn {
	os::exit_with(foo());
}

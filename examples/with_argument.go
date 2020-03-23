require "os";
func foo(x int, y int) int {
	return x + y; 
}
func main() noreturn {
    os::exit_with(foo(10, 20));
}

require (
    "std"
)
func foo(x int64, y int64) int64 {
	return x + y; 
}
func main() noreturn {
    std::os::exit_with(foo(10, 20));
}

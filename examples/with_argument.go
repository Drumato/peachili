require (
    "std"
)
func foo(x Int64, y Int64) Int64 {
	return x + y; 
}
func main() Noreturn {
    std::os::exit_with(foo(10, 20));
}

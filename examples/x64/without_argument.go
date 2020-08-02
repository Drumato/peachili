import x64;

func foo() Int64 {
	return 30; 
}
func main() Noreturn {
	x64::exit_with(foo());
}

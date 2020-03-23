require (
    "os"
)
func main() noreturn {
	declare x int;
	declare y int;
	x = y = 3;
	os::exit_with(x * y);
}

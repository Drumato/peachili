require (
    "os"
)
func main() noreturn {
	declare x int; 
	x = if (0) { 
		ifret 1; 
	} else { 
		ifret 0; 
	}; 
	os::exit_with(x);
}

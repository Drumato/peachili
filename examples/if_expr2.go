require (
    "std"
)
func main() noreturn {
	declare x int; 
	x = if (0) { 
		ifret 1; 
	} else { 
		ifret 0; 
	}; 
	std::os::exit_with(x);
}

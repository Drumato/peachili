require (
    "std"
)
func main() noreturn {
	declare x int64;

	// if式内の条件式はboolean型のみ
	x = if (0) { 
		ifret 1; 
	} else { 
		ifret 0; 
	}; 
	std::os::exit_with(x);
}

func main() int { 
	declare x int; 
	x = if (1) { 
		ifret 1; 
	} else { 
		ifret 0; 
	}; 
	return x;
}

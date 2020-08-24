import x64;

func main() Noreturn {
    x64::write(1u, "Hello, world!\n", 14);
    x64::exit_with(0);
}

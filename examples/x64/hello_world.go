import x64;

func main() Noreturn {
    x64::write(x64::STDOUT, "Hello, world!\n", 14);
    x64::exit_with(0);
}

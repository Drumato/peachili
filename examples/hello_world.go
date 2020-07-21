import std;

func main() Noreturn {
    std::os::write(1u, "Hello, world!\n", 15);
    std::os::exit_with(0);
}

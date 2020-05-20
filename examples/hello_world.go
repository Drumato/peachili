require (
    "std"
)

func main() noreturn {
    std::os::write(1u, "Hello, world!\n", 15);
    std::os::exit_with(0);
}
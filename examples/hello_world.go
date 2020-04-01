require (
    "std"
)

func main() noreturn {
    std::os::write(1, "Hello, world!\n", 14);
}
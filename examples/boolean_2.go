require (
    "std"
)

func main() noreturn {
    if (false) {
        std::os::exit_with(15);
    } else {
        std::os::exit_with(30);
    };
}
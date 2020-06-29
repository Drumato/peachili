require (
    "std"
)

func main() Noreturn {
    if (false) {
        std::os::exit_with(15);
    } else {
        std::os::exit_with(30);
    };
}
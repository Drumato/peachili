import std;
func main() Noreturn {
    if (true) {
        std::os::exit_with(15);
    } else {
        std::os::exit_with(30);
    };
}

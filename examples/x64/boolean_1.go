import x64;

func main() Noreturn {
    if (true) {
        x64::exit_with(15);
    } else {
        x64::exit_with(30);
    };
}

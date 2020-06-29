require (
    "std"
)

func main() Noreturn {
    declare x Uint64;
    x = 100u;
    std::os::exit_with(1);
}
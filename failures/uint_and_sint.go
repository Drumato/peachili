require (
    "std"
)

func main() Noreturn {
    declare x Uint64;
    declare y Int64;
    x = 100;
    y = 50u;

    declare z Int64;
    z = x + y;
    std::os::exit_with(z);
}

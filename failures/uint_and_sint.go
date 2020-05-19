require (
    "std"
)

func main() noreturn {
    declare x uint64;
    declare y int64;
    x = 100;
    y = 50u;

    declare z int64;
    z = x + y;
    std::os::exit_with(z);
}
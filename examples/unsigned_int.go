require (
    "std"
)

func main() noreturn {
    declare x uint64;
    x = 100u;
    std::os::exit_with(1);
}
require (
    "std"
)

func main() noreturn {
    declare x int64;
    x = 30 + true;
    std::os::exit_with(2);
}
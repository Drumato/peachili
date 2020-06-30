require (
    "std"
)

func main() Noreturn {
    declare x Int64;
    x = 30 + true;
    std::os::exit_with(2);
}

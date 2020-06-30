require (
    "std"
)

func main() Noreturn {
    varinit x Int64 = 30;
    const y Int64 = x;
    y = y + 2;

    std::os::exit_with(0);
}

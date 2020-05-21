require (
    "std"
)

func main() noreturn {
    varinit x int64 = 30;
    const y int64 = x;
    y = y + 2;

    std::os::exit_with(0);
}
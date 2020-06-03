require (
    "std"
)
func main() noreturn {
    varinit x int64 = 0;
    varinit y *int64 = &x;
    *y = 4;

    std::os::exit_with(x);
}

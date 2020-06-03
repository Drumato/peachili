require (
    "std"
)
func main() noreturn {
    varinit x int64 = 4;
    varinit y *int64 = &x;
    std::os::exit_with(*y);
}

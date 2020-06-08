require (
    "std"
)

pubtype PointerToI64 = *int64;

func main() noreturn {
    varinit x int64 = 0;
    varinit y PointerToI64 = &x;
    *y = 4;

    std::os::exit_with(x);
}

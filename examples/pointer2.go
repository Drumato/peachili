import std;

pubtype PointerToI64 = *Int64;

func main() Noreturn {
    varinit x Int64 = 0;
    varinit y PointerToI64 = &x;
    *y = 4;

    std::os::exit_with(x);
}

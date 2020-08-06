import x64;

pubtype PointerToI64 = *Int64;

func main() Noreturn {
    varinit x Int64 = 0;
    varinit y PointerToI64 = &x;
    *y = 4;

    x64::exit_with(x);
}

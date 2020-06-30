func main() Noreturn {
    callee();
}

func callee() *Int64 {
    varinit x Int64 = 0;
    return &x;
}

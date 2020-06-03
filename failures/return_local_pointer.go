func main() noreturn {
    callee();
}

func callee() *int64 {
    varinit x int64 = 0;
    return &x;
}
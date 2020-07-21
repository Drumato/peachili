import std;

func main() Noreturn {
    varinit x Int64 = 4;
    varinit y *Int64 = &x;
    std::os::exit_with(*y);
}

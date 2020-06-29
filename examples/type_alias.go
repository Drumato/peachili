require (
    "std"
)

pubtype Another = Int64;

func main() Noreturn {
    declare x Another;
    x = 30;
    std::os::exit_with(x);
}
require (
    "std"
)

pubtype Another = int64;

func main() noreturn {
    declare x Another;
    x = 30;
    std::os::exit_with(x);
}
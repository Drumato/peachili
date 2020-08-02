import x64;

pubtype Another = Int64;

func main() Noreturn {
    declare x Another;
    x = 30;
    x64::exit_with(x);
}

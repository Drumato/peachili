require (
    "os"
    "version"
)
func main() noreturn {
    os::exit_with(version::peachili_version());
}
require (
    "std"
)

func main() Noreturn {
	declare res Int64;

	// 左閉右開区間の数え上げ
	countup x Int64 from 0 to 10 {
      		res = x;
    };
	std::os::exit_with(res);
}

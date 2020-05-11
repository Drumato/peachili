require (
    "std"
)

func main() noreturn {
	declare res int64;

	// 左閉右開区間の数え上げ
	countup x int64 from 0 to 10 {
      		res = x;
    };
	std::os::exit_with(res);
}

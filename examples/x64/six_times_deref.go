import x64;

func main() Noreturn {
	varinit a Int64 = 4;
	varinit b *Int64 = &a;
	varinit c **Int64 = &b;
	varinit d ***Int64 = &c;
	varinit e ****Int64 = &d;
	varinit f *****Int64 = &e;
	x64::exit_with(*****f);
}

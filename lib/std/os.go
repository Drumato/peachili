func exit_with(status int64) noreturn {
    asm {
        "movq $60, %rax",
        "syscall"
    };
}

func write(fd int64, buf str, count int64) noreturn {
    asm {
        "movq $1, %rax",
        "syscall"
    };
}

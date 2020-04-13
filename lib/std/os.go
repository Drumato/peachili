func exit_with(status int) noreturn {
  asm {
    "movq $60, %rax",
    "syscall"
  };
}

func write(fd int, buf str, count int) noreturn {
    asm {
        "movq $1, %rax",
        "syscall"
    };
}

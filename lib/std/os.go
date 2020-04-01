func exit_with(status int) noreturn {
  asm {
    "mov rax, 60",
    "syscall"
  };
}

func write(fd int, buf str, count int) noreturn {
    asm {
        "mov rax, 1",
        "syscall"
    };
}

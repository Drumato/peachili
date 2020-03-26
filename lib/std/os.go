func exit_with(status int) int {
  asm {
    "mov rax, 60",
    "syscall"
  };
}
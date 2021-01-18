func initialize() Noreturn {
    asm {
        "call main";
        "mov rdi, rax"; 
        "mov rax, 60";
        "syscall"
    };
}

func initialize() Noreturn {
    asm {
        "call main";
        "movq %rax, %rdi"; 
        "movq $60, %rax";
        "syscall"
    };
}

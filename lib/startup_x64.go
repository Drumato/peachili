func initialize() noreturn {
    asm {
        "call main",
        "movq %rax, %rdi", // main関数の返り値(通常は0)をプロセス全体の返り値に
        "movq $60, %rax", // 64bit linuxにおけるexitシステムコール
        "syscall"
    };
}
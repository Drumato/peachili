func initialize() noreturn {
    asm {
        "bl main",
        "mov x0, x1", // main関数の返り値(通常は0)をプロセス全体の返り値に
        "mov x8, #93", // 64bit linuxにおけるexitシステムコール
        "svc #0"
    };
}

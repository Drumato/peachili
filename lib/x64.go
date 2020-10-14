func exit_with(status Int64) Noreturn {
    asm {
        "movq $60, %rax";
        "syscall";
    };
}

func write(fd FileDescriptor, buf ConstStr, count Int64) Noreturn {
    asm {
        "movq $1, %rax";
        "syscall";
    };
}

pubtype FileDescriptor = Uint64;

pubconst STDIN : FileDescriptor = 0u;
pubconst STDOUT : FileDescriptor = 1u;
pubconst STDERR : FileDescriptor = 2u;

func exit_with(status Int64) Noreturn {
    asm {
        "mov rax, 60";
        "syscall"
    };
}

func write(fd FileDescriptor, buf ConstStr, count Int64) Noreturn {
    asm {
        "mov rax, 1";
        "syscall"
    };
}

pubtype FileDescriptor = Uint64;

pubconst STDIN : FileDescriptor = u0;
pubconst STDOUT : FileDescriptor = u1;
pubconst STDERR : FileDescriptor = u2;

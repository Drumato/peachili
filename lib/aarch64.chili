func exit_with(status Int64) Noreturn {
    asm {
        "mov x8, #93";
        "svc #0"
    };
}

func write(fd FileDescriptor, buf ConstStr, count Int64) Noreturn {
    asm {
        "mov x8, #64";
        "svc #0"
    };
}

pubtype FileDescriptor = Uint64;

pubconst STDIN : FileDescriptor = u0;
pubconst STDOUT : FileDescriptor = u1;
pubconst STDERR : FileDescriptor = u2;

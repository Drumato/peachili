func exit_with(status Int64) Noreturn {
     asm {
			 "mov x8, #93"; // 64bit linuxにおけるexitシステムコール
			 "svc #0"
     };
 }

func write(fd FileDescriptor, buf ConstStr, count Int64) Noreturn {
    asm {
			 "mov x8, #64"; // 64bit linuxにおけるwriteシステムコール
			 "svc #0"
    };
}

pubtype FileDescriptor = Uint64;

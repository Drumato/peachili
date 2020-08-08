.global "main"
"main":
  "main_entry":
    pushq %rbp
    movq %rsp, %rbp
    subq $56, %rsp
    movq $0, -8(%rbp)
    leaq -8(%rbp), %r10
    movq %r10, -16(%rbp)
    leaq -16(%rbp), %r11
    movq %r11, -24(%rbp)
    leaq -24(%rbp), %r12
    movq %r12, -32(%rbp)
    leaq -32(%rbp), %r13
    movq %r13, -40(%rbp)
    leaq -40(%rbp), %r14
    movq %r14, -48(%rbp)
    leaq -48(%rbp), %r15
    movq %r15, -56(%rbp)
    leaq -56(%rbp), %r10
    movq %r10, %r11
    movq (%r11), %r11
    movq %r11, %r12
    movq (%r12), %r12
    movq %r12, %r13
    movq (%r13), %r13
    movq %r13, %r14
    movq (%r14), %r14
    movq %r14, %r15
    movq (%r15), %r15
    movq %r15, %r10
    movq (%r10), %r10
    movq $4, (%r10)
    movq -8(%rbp), %rdi
    call "x64::exit_with"
    movq %rax, %r11

.global "x64::exit_with"
"x64::exit_with":
  "x64::exit_with_entry":
    pushq %rbp
    movq %rsp, %rbp
    subq $8, %rsp
    movq %rdi, -8(%rbp)
    movq $60, %rax
    syscall

.global "x64::write"
"x64::write":
  "x64::write_entry":
    pushq %rbp
    movq %rsp, %rbp
    subq $24, %rsp
    movq %rdi, -24(%rbp)
    movq %rsi, -8(%rbp)
    movq %rdx, -16(%rbp)
    movq $1, %rax
    syscall


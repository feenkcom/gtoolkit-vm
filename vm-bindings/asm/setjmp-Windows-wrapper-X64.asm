; Microsoft Assembler (MASM) implementation of the __setjmp_wrapper and __longjmp_wrapper
; https://docs.microsoft.com/en-us/cpp/assembler/masm/masm-for-x64-ml64-exe?view=msvc-160
.code

__setjmp_wrapper PROC
    mov    rdx,QWORD PTR [rsp] ; rta
    mov    rax,QWORD PTR gs:[0] ; SEH
    mov    QWORD PTR [rcx+0],rax
    mov    QWORD PTR [rcx+8],rbx
    mov    QWORD PTR [rcx+16],rsp
    mov    QWORD PTR [rcx+24],rbp
    mov    QWORD PTR [rcx+32],rsi
    mov    QWORD PTR [rcx+40],rdi
    mov    QWORD PTR [rcx+48],r12
    mov    QWORD PTR [rcx+56],r13
    mov    QWORD PTR [rcx+64],r14
    mov    QWORD PTR [rcx+72],r15
    mov    QWORD PTR [rcx+80],rdx ; rip
    mov    QWORD PTR [rcx+88],0
    movaps [rcx+96],xmm6
    movaps [rcx+112],xmm7
    movaps [rcx+128],xmm8
    movaps [rcx+144],xmm9
    movaps [rcx+160],xmm10
    movaps [rcx+176],xmm11
    movaps [rcx+192],xmm12
    movaps [rcx+208],xmm13
    movaps [rcx+224],xmm14
    movaps [rcx+240],xmm15
    xor    rax,rax ; return 0
    ret
__setjmp_wrapper ENDP

__longjmp_wrapper PROC
    mov    rax,QWORD PTR [rcx+0]
    mov    rbx,QWORD PTR [rcx+8]
    mov    rsp,QWORD PTR [rcx+16]
    mov    rbp,QWORD PTR [rcx+24]
    mov    rsi,QWORD PTR [rcx+32]
    mov    rdi,QWORD PTR [rcx+40]
    mov    r12,QWORD PTR [rcx+48]
    mov    r13,QWORD PTR [rcx+56]
    mov    r14,QWORD PTR [rcx+64]
    mov    r15,QWORD PTR [rcx+72]
    mov    r8, QWORD PTR [rcx+80]
    movaps xmm6,[rcx+96]
    movaps xmm7,[rcx+112]
    movaps xmm8,[rcx+128]
    movaps xmm9,[rcx+144]
    movaps xmm10,[rcx+160]
    movaps xmm11,[rcx+176]
    movaps xmm12,[rcx+192]
    movaps xmm13,[rcx+208]
    movaps xmm14,[rcx+224]
    movaps xmm15,[rcx+240]
    mov    QWORD PTR gs:[0],rax
    mov    eax,edx ; move arg2 to return
    test   eax,eax
    jne    a
    inc    eax
a:  mov    QWORD PTR [rsp],r8
    ret
__longjmp_wrapper ENDP

END
.section .rodata.user,"a"
.global user_hello_start,user_hello_end
user_hello_start:
    mov eax,1
    lea rdi,[rip+hello_message]
    mov esi,12
    int 0x80
    mov eax,2
    int 0x80
    ud2
hello_message: .ascii "hello:world\n"
user_hello_end:
.global user_count_start,user_count_end
user_count_start:
    mov eax,4
    int 0x80
    mov r13,rax
count_loop:
    pause
    mov eax,4
    int 0x80
    sub rax,r13
    cmp rax,20
    jb count_loop
    mov eax,1
    lea rdi,[rip+count_message]
    mov esi,11
    int 0x80
    mov eax,2
    int 0x80
    ud2
count_message: .ascii "count:done\n"
user_count_end:
.global user_fault_start,user_fault_end
user_fault_start:
    movabs rax,0xffffffff80000000
    mov rax,[rax]
    ud2
user_fault_end:
.global user_writefault_start,user_writefault_end
user_writefault_start:
    movabs rax,0x400000
    mov byte ptr [rax],0
    ud2
user_writefault_end:
.global user_badptr_start,user_badptr_end
user_badptr_start:
    mov eax,1
    movabs rdi,0xffffffff80000000
    mov esi,8
    int 0x80
    cmp rax,-14
    jne badptr_fail
    mov eax,1
    mov rdi,-2
    mov esi,4
    int 0x80
    cmp rax,-14
    jne badptr_fail
    mov eax,1
    mov edi,0x400fff
    mov esi,2
    int 0x80
    cmp rax,-14
    jne badptr_fail
    mov eax,1
    mov edi,0x400000
    mov esi,257
    int 0x80
    cmp rax,-14
    jne badptr_fail
    mov eax,1
    lea rdi,[rip+badptr_message]
    mov esi,10
    int 0x80
    mov eax,2
    int 0x80
badptr_fail: ud2
badptr_message: .ascii "badptr:ok\n"
user_badptr_end:
.global user_sender_start,user_sender_end
user_sender_start:
    mov eax,5
    mov rdi,r12
    mov rsi,-1
    int 0x80
    test rax,rax
    jne sender_fail
    mov eax,5
    mov rdi,r12
    mov esi,42
    int 0x80
    test rax,rax
    jne sender_fail
    mov eax,1
    lea rdi,[rip+sender_message]
    mov esi,9
    int 0x80
    mov eax,2
    int 0x80
sender_fail: ud2
sender_message: .ascii "ipc:sent\n"
user_sender_end:
.global user_receiver_start,user_receiver_end
user_receiver_start:
    mov eax,6
    int 0x80
    test rdx,rdx
    jz receiver_value
    mov eax,3
    int 0x80
    jmp user_receiver_start
receiver_value:
    cmp rax,-1
    jne receiver_fail
receiver_second:
    mov eax,6
    int 0x80
    test rdx,rdx
    jz receiver_second_value
    mov eax,3
    int 0x80
    jmp receiver_second
receiver_second_value:
    cmp rax,42
    jne receiver_fail
    mov eax,1
    lea rdi,[rip+receiver_message]
    mov esi,13
    int 0x80
    mov eax,2
    int 0x80
receiver_fail: ud2
receiver_message: .ascii "ipc:received\n"
user_receiver_end:
.global user_spin_start,user_spin_end
user_spin_start:
    pause
    jmp user_spin_start
user_spin_end:

.global user_fpu_start,user_fpu_end
user_fpu_start:
    fld1
    ud2
user_fpu_end:

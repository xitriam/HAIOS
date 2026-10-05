.section .text
.global isr_0
isr_0:
    push 0
    push 0
    jmp interrupt_common
.global isr_1
isr_1:
    push 0
    push 1
    jmp interrupt_common
.global isr_2
isr_2:
    push 0
    push 2
    jmp interrupt_common
.global isr_3
isr_3:
    push 0
    push 3
    jmp interrupt_common
.global isr_4
isr_4:
    push 0
    push 4
    jmp interrupt_common
.global isr_5
isr_5:
    push 0
    push 5
    jmp interrupt_common
.global isr_6
isr_6:
    push 0
    push 6
    jmp interrupt_common
.global isr_7
isr_7:
    push 0
    push 7
    jmp interrupt_common
.global isr_8
isr_8:
    push 8
    jmp interrupt_common
.global isr_9
isr_9:
    push 0
    push 9
    jmp interrupt_common
.global isr_10
isr_10:
    push 10
    jmp interrupt_common
.global isr_11
isr_11:
    push 11
    jmp interrupt_common
.global isr_12
isr_12:
    push 12
    jmp interrupt_common
.global isr_13
isr_13:
    push 13
    jmp interrupt_common
.global isr_14
isr_14:
    push 14
    jmp interrupt_common
.global isr_15
isr_15:
    push 0
    push 15
    jmp interrupt_common
.global isr_16
isr_16:
    push 0
    push 16
    jmp interrupt_common
.global isr_17
isr_17:
    push 17
    jmp interrupt_common
.global isr_18
isr_18:
    push 0
    push 18
    jmp interrupt_common
.global isr_19
isr_19:
    push 0
    push 19
    jmp interrupt_common
.global isr_20
isr_20:
    push 0
    push 20
    jmp interrupt_common
.global isr_21
isr_21:
    push 21
    jmp interrupt_common
.global isr_22
isr_22:
    push 0
    push 22
    jmp interrupt_common
.global isr_23
isr_23:
    push 0
    push 23
    jmp interrupt_common
.global isr_24
isr_24:
    push 0
    push 24
    jmp interrupt_common
.global isr_25
isr_25:
    push 0
    push 25
    jmp interrupt_common
.global isr_26
isr_26:
    push 0
    push 26
    jmp interrupt_common
.global isr_27
isr_27:
    push 0
    push 27
    jmp interrupt_common
.global isr_28
isr_28:
    push 0
    push 28
    jmp interrupt_common
.global isr_29
isr_29:
    push 29
    jmp interrupt_common
.global isr_30
isr_30:
    push 30
    jmp interrupt_common
.global isr_31
isr_31:
    push 0
    push 31
    jmp interrupt_common
.global isr_32
isr_32:
    push 0
    push 32
    jmp interrupt_common
.global isr_33
isr_33:
    push 0
    push 33
    jmp interrupt_common
.global isr_34
isr_34:
    push 0
    push 34
    jmp interrupt_common
.global isr_35
isr_35:
    push 0
    push 35
    jmp interrupt_common
.global isr_36
isr_36:
    push 0
    push 36
    jmp interrupt_common
.global isr_37
isr_37:
    push 0
    push 37
    jmp interrupt_common
.global isr_38
isr_38:
    push 0
    push 38
    jmp interrupt_common
.global isr_39
isr_39:
    push 0
    push 39
    jmp interrupt_common
.global isr_40
isr_40:
    push 0
    push 40
    jmp interrupt_common
.global isr_41
isr_41:
    push 0
    push 41
    jmp interrupt_common
.global isr_42
isr_42:
    push 0
    push 42
    jmp interrupt_common
.global isr_43
isr_43:
    push 0
    push 43
    jmp interrupt_common
.global isr_44
isr_44:
    push 0
    push 44
    jmp interrupt_common
.global isr_45
isr_45:
    push 0
    push 45
    jmp interrupt_common
.global isr_46
isr_46:
    push 0
    push 46
    jmp interrupt_common
.global isr_47
isr_47:
    push 0
    push 47
    jmp interrupt_common
.global isr_128
isr_128:
    push 0
    push 128
    jmp interrupt_common
interrupt_common:
    cld
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rdi
    push rsi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    mov rdi, rsp
    mov rbx, rsp
    and rsp, -16
    call interrupt_dispatch
    mov rsp, rbx
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rsi
    pop rdi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax
    add rsp, 16
    iretq
.global reload_segments
reload_segments:
    push 8
    lea rax, [rip + 1f]
    push rax
    retfq
1:
    mov ax, 16
    mov ds, ax
    mov es, ax
    mov ss, ax
    ret
.global kernel_idle
kernel_idle:
    sti
2:
    hlt
    jmp 2b
.section .rodata
.global isr_table
isr_table:
    .quad isr_0
    .quad isr_1
    .quad isr_2
    .quad isr_3
    .quad isr_4
    .quad isr_5
    .quad isr_6
    .quad isr_7
    .quad isr_8
    .quad isr_9
    .quad isr_10
    .quad isr_11
    .quad isr_12
    .quad isr_13
    .quad isr_14
    .quad isr_15
    .quad isr_16
    .quad isr_17
    .quad isr_18
    .quad isr_19
    .quad isr_20
    .quad isr_21
    .quad isr_22
    .quad isr_23
    .quad isr_24
    .quad isr_25
    .quad isr_26
    .quad isr_27
    .quad isr_28
    .quad isr_29
    .quad isr_30
    .quad isr_31
    .quad isr_32
    .quad isr_33
    .quad isr_34
    .quad isr_35
    .quad isr_36
    .quad isr_37
    .quad isr_38
    .quad isr_39
    .quad isr_40
    .quad isr_41
    .quad isr_42
    .quad isr_43
    .quad isr_44
    .quad isr_45
    .quad isr_46
    .quad isr_47

.section .text.entry,"ax"
.global _start
_start:
    cli
    cld
    and rsp, -16
    call kernel_main
    ud2
.global haios_halt
haios_halt:
    cli
1:
    hlt
    jmp 1b
.global haios_halt_end
haios_halt_end:

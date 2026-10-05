// Position-independent one-page processes. Own stack holds arguments and scratch.
.section .rodata.user,"a",@progbits
.global user_echo_start,user_echo_end
user_echo_start:
 sub rsp,272
 mov rdx,rsi
 cmp rdx,255
 jbe .Lecho_length
 mov edx,255
.Lecho_length:
 mov rcx,rdx
 mov rsi,rdi
 mov rdi,rsp
 rep movsb
 mov byte ptr [rsp+rdx],10
 mov rdi,rsp
 lea rsi,[rdx+1]
 mov eax,1
 int 0x80
 mov eax,2
 int 0x80
 ud2
user_echo_end:

.global user_uptime_start,user_uptime_end
user_uptime_start:
 mov eax,4
 int 0x80
 xor edx,edx
 mov ecx,100
 div rcx
 lea rdi,[rsp-32]
 mov byte ptr [rdi],10
 mov esi,1
 mov ecx,10
.Luptime_digit:
 xor edx,edx
 div rcx
 add dl,48
 dec rdi
 mov [rdi],dl
 inc rsi
 test rax,rax
 jnz .Luptime_digit
 mov eax,1
 int 0x80
 mov eax,2
 int 0x80
 ud2
user_uptime_end:

.global user_wc_start,user_wc_end
user_wc_start:
 mov r12,rdi
 mov r13,rsi
 xor r14d,r14d // offset/bytes
 xor r15d,r15d // lines
 xor ebp,ebp // words
 xor ebx,ebx // in word
 sub rsp,512
.Lwc_read:
 mov eax,7
 mov rdi,r12
 mov rsi,r13
 mov rdx,rsp
 mov r10d,256
 mov r8,r14
 int 0x80
 test rax,rax
 js .Lwc_error
 jz .Lwc_done
 add r14,rax
 xor ecx,ecx
.Lwc_scan:
 mov dl,[rsp+rcx]
 cmp dl,10
 jne .Lwc_space
 inc r15
.Lwc_space:
 cmp dl,32
 ja .Lwc_word
 xor ebx,ebx
 jmp .Lwc_next
.Lwc_word:
 test ebx,ebx
 jnz .Lwc_next
 inc rbp
 mov ebx,1
.Lwc_next:
 inc rcx
 cmp rcx,rax
 jb .Lwc_scan
 jmp .Lwc_read
.Lwc_done:
 // Output lines words bytes as a single buffer.
 lea rdi,[rsp+480]
 mov byte ptr [rdi],10
 mov esi,1
 mov rax,r14
 call .Lwc_number
 dec rdi
 mov byte ptr [rdi],32
 inc rsi
 mov rax,rbp
 call .Lwc_number
 dec rdi
 mov byte ptr [rdi],32
 inc rsi
 mov rax,r15
 call .Lwc_number
 mov eax,1
 int 0x80
 jmp .Lwc_exit
.Lwc_number:
 mov ecx,10
.Lwc_digit:
 xor edx,edx
 div rcx
 add dl,48
 dec rdi
 mov [rdi],dl
 inc rsi
 test rax,rax
 jnz .Lwc_digit
 ret
.Lwc_error:
 lea rdi,[rip+.Lwc_message]
 mov esi,19
 mov eax,1
 int 0x80
.Lwc_exit:
 mov eax,2
 int 0x80
 ud2
.Lwc_message:.ascii "ERROR wc: readfile\n"
.Lwc_message_end:
user_wc_end:

.global user_calc_start,user_calc_end
user_calc_start:
 mov r12,rdi
 lea r13,[rdi+rsi]
 call .Lcalc_parse
 jc .Lcalc_error
 mov r14,rax
 call .Lcalc_spaces
 cmp r12,r13
 jae .Lcalc_error
 mov bl,[r12]
 inc r12
 call .Lcalc_parse
 jc .Lcalc_error
 mov r15,rax
 call .Lcalc_spaces
 cmp r12,r13
 jne .Lcalc_error
 mov rax,r14
 cmp bl,43
 je .Lcalc_add
 cmp bl,45
 je .Lcalc_sub
 cmp bl,42
 je .Lcalc_mul
 cmp bl,47
 jne .Lcalc_error
 test r15,r15
 jz .Lcalc_error
 mov rcx,0x8000000000000000
 cmp rax,rcx
 jne .Lcalc_div
 cmp r15,-1
 je .Lcalc_error
.Lcalc_div:
 cqo
 idiv r15
 jmp .Lcalc_print
.Lcalc_add:
 add rax,r15
 jo .Lcalc_error
 jmp .Lcalc_print
.Lcalc_sub:
 sub rax,r15
 jo .Lcalc_error
 jmp .Lcalc_print
.Lcalc_mul:
 imul rax,r15
 jo .Lcalc_error
.Lcalc_print:
 lea rdi,[rsp-32]
 mov byte ptr [rdi],10
 mov esi,1
 xor r8d,r8d
 test rax,rax
 jns .Lcalc_unsigned
 mov r8d,1
 neg rax // MIN stays unsigned magnitude, intentionally
.Lcalc_unsigned:
 mov ecx,10
.Lcalc_digit:
 xor edx,edx
 div rcx
 add dl,48
 dec rdi
 mov [rdi],dl
 inc rsi
 test rax,rax
 jnz .Lcalc_digit
 test r8,r8
 jz .Lcalc_output
 dec rdi
 mov byte ptr [rdi],45
 inc rsi
.Lcalc_output:
 mov eax,1
 int 0x80
 jmp .Lcalc_exit
.Lcalc_error:
 lea rdi,[rip+.Lcalc_message]
 mov esi,49
 mov eax,1
 int 0x80
.Lcalc_exit:
 mov eax,2
 int 0x80
 ud2
.Lcalc_spaces:
 cmp r12,r13
 jae .Lcalc_spaces_done
 cmp byte ptr [r12],32
 jne .Lcalc_spaces_done
 inc r12
 jmp .Lcalc_spaces
.Lcalc_spaces_done:
 ret
.Lcalc_parse:
 call .Lcalc_spaces
 xor eax,eax
 xor r9d,r9d // negative sign
 xor r10d,r10d // digits
 cmp r12,r13
 jae .Lcalc_parse_bad
 cmp byte ptr [r12],45
 jne .Lcalc_plus
 mov r9d,1
 inc r12
 jmp .Lcalc_parse_digit
.Lcalc_plus:
 cmp byte ptr [r12],43
 jne .Lcalc_parse_digit
 inc r12
.Lcalc_parse_digit:
 cmp r12,r13
 jae .Lcalc_parse_end
 movzx edx,byte ptr [r12]
 sub edx,48
 cmp edx,9
 ja .Lcalc_parse_end
 imul rax,rax,10
 jo .Lcalc_parse_bad
 sub rax,rdx
 jo .Lcalc_parse_bad
 inc r12
 inc r10
 jmp .Lcalc_parse_digit
.Lcalc_parse_end:
 test r10,r10
 jz .Lcalc_parse_bad
 test r9,r9
 jnz .Lcalc_parse_ok
 neg rax
 jo .Lcalc_parse_bad
.Lcalc_parse_ok:
 clc
 ret
.Lcalc_parse_bad:
 stc
 ret
.Lcalc_message:.ascii "ERROR calc: syntax, overflow or division by zero\n"
.Lcalc_message_end:
user_calc_end:

// Regression process probes the readfile trust boundary, never dereferences bad pointers.
.global user_fsbadptr_start,user_fsbadptr_end
user_fsbadptr_start:
 sub rsp,512
 lea rdi,[rip+.Lfs_path]
 mov esi,6
 mov rdx,rsp
 mov r10d,16
 xor r8d,r8d
 mov eax,7
 int 0x80
 test rax,rax
 jle .Lfs_fail
 lea rdi,[rip+.Lfs_path]
 mov esi,6
 lea rdx,[rip+.Lfs_path] // own RX code cannot be an output
 mov r10d,16
 xor r8d,r8d
 mov eax,7
 int 0x80
 cmp rax,-14
 jne .Lfs_fail
 lea rdi,[rip+.Lfs_path]
 mov esi,6
 mov rdx,0xffffffff80000000 // supervisor pointer
 mov r10d,16
 mov eax,7
 int 0x80
 cmp rax,-14
 jne .Lfs_fail
 lea rdi,[rip+.Lfs_path]
 mov esi,6
 mov rdx,0x800fff // crosses own stack page
 mov r10d,16
 mov eax,7
 int 0x80
 cmp rax,-14
 jne .Lfs_fail
 lea rdi,[rip+.Lfs_path]
 mov esi,6
 mov rdx,-4 // integer overflow
 mov r10d,16
 mov eax,7
 int 0x80
 cmp rax,-14
 jne .Lfs_fail
 lea rdi,[rip+.Lfs_path]
 mov esi,6
 mov rdx,rsp
 mov r10d,257
 mov eax,7
 int 0x80
 cmp rax,-22
 jne .Lfs_fail
 mov rdi,0xffffffff80000000
 mov esi,6
 mov rdx,rsp
 mov r10d,16
 mov eax,7
 int 0x80
 cmp rax,-14
 jne .Lfs_fail
 lea rdi,[rip+.Lfs_path]
 mov esi,6
 mov rdx,rsp
 mov r10d,16
 mov r8,-1 // out of bounds file offset
 mov eax,7
 int 0x80
 cmp rax,-22
 jne .Lfs_fail
 lea rdi,[rip+.Lfs_ok]
 mov esi,12
 jmp .Lfs_output
.Lfs_fail:
 lea rdi,[rip+.Lfs_bad]
 mov esi,14
.Lfs_output:
 mov eax,1
 int 0x80
 mov eax,2
 int 0x80
 ud2
.Lfs_path:.ascii "/about"
.Lfs_ok:.ascii "fsbadptr:ok\n"
.Lfs_bad:.ascii "fsbadptr:FAIL\n"
user_fsbadptr_end:

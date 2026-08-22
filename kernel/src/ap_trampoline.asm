bits 16
org 0

; SIPI starts at offset zero. Jump over the patch slots so Rust can fill
; physical CR3, stack, CPU id, and long-mode entry without rewriting code.
jmp start

times 16 - ($ - $$) db 0
pml4_slot:  dq 0x504d4c345f534c54
stack_slot: dq 0x535441434b534c54
cpu_slot:   dq 0x4350555f534c4f54
entry_slot: dq 0x454e545259534c54
long_entry_slot: dq 0x4c4f4e475f534c54
online_slot:     dq 0x4f4e4c494e45534c
idt_slot:        dq 0x4944545f534c4f54
stage_slot:      dq 0x53544147455f534c
per_cpu_slot:    dq 0x504552435055534c

times 96 - ($ - $$) db 0

start:
    cli
    mov ax, cs
    mov ds, ax
    mov ss, ax
    mov sp, 0x1000

    ; Fast A20 gate. QEMU and modern firmware expose port 0x92.
    in al, 0x92
    or al, 2
    and al, 0xfe
    out 0x92, al

    mov ax, cs
    movzx eax, ax
    shl eax, 4
    add dword [gdt16 + 2], eax
    ; Protected-mode descriptors must address this physical trampoline page.
    mov word [gdt + 8 + 2], ax
    mov word [gdt + 16 + 2], ax
    shr eax, 16
    mov byte [gdt + 8 + 4], al
    mov byte [gdt + 16 + 4], al
    mov byte [gdt + 8 + 7], ah
    mov byte [gdt + 16 + 7], ah
    ; Long-mode code selector (0x18) must retain a zero segment base.
    mov word [gdt + 24 + 2], 0
    mov byte [gdt + 24 + 4], 0
    mov byte [gdt + 24 + 7], 0
    lgdt [gdt16]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp 0x08:protected_mode

bits 32
protected_mode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    mov eax, cr4
    or eax, (1 << 5)              ; PAE
    mov cr4, eax

    mov eax, dword [pml4_slot]
    mov cr3, eax

    mov ecx, 0xc0000080            ; IA32_EFER
    rdmsr
    or eax, (1 << 8)               ; LME
    wrmsr

    mov byte [0x3f0], 1
    mov eax, dword [long_entry_slot]
    mov ebx, eax
    mov ecx, cr0
    or ecx, 0x80000000             ; PG
    mov cr0, ecx
    push dword 0x18
    push ebx
    retf

bits 64
default rel
 times 0x180 - ($ - $$) db 0
long_mode:
    ; Keep the first AP proof entirely in the low identity-mapped page.
    ; Rust code and high-half kernel data are intentionally not touched until
    ; per-CPU page-table coverage has been proven separately.
    mov rax, qword [rel online_slot]
    mov byte [rax], 3
    mov rax, qword [rel stage_slot]
    mov byte [rax], 1
%ifndef LOW_HANDSHAKE
    mov rsp, qword [rel stack_slot]
    mov rdi, qword [rel cpu_slot]
    mov rsi, qword [rel stack_slot]
    mov rdx, qword [rel idt_slot]
    mov rcx, qword [rel stage_slot]
    mov r8, qword [rel per_cpu_slot]
    mov rax, qword [rel entry_slot]
    call rax
%endif

halt:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    cli
    hlt
    jmp halt

align 8
gdt16:
    dw gdt_end - gdt - 1
    dd gdt

gdt:
    dq 0
    ; 32-bit flat code: base 0, limit 4 GiB, present, ring 0, executable.
    dw 0xffff, 0
    db 0, 0x9a, 0xcf, 0
    ; Flat data.
    dw 0xffff, 0
    db 0, 0x92, 0xcf, 0
    ; 64-bit code: base 0, long-mode bit set.
    dw 0, 0
    db 0, 0x9a, 0xaf, 0
gdt_end:

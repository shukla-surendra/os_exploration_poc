; boot.asm - minimal boot sector demo
org 0x7C00              ; BIOS loads boot sector here

start:
    cli                 ; clear interrupts (safe)
    xor ax, ax
    mov ss, ax
    mov sp, 0x7C00      ; simple stack
    sti

    mov si, msg
.print_char:
    lodsb               ; load byte from DS:SI into AL, inc SI
    cmp al, 0
    je .hang
    mov ah, 0x0E        ; BIOS teletype function
    mov bh, 0x00
    mov bl, 0x07
    int 0x10
    jmp .print_char

.hang:
    cli
    hlt
    jmp .hang

msg: db "Hello from boot sector! Press power to turn off.", 0

; pad up to 510 bytes; last two bytes are 0x55 0xAA
times 510-($-$$) db 0
dw 0xAA55

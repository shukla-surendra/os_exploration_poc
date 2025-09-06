; hello.asm - Hello World in Linux x86 Assembly (32-bit)

section .data
    msg db "Hello, World!", 0xA  ; string with newline
    len equ $ - msg              ; length of string

section .text
    global _start

_start:
    ; sys_write(fd=1, buf=msg, count=len)
    mov eax, 4          ; syscall number (sys_write)
    mov ebx, 1          ; file descriptor (stdout)
    mov ecx, msg        ; pointer to message
    mov edx, len        ; message length
    int 0x80            ; call kernel

    ; sys_exit(status=0)
    mov eax, 1          ; syscall number (sys_exit)
    xor ebx, ebx        ; exit code 0
    int 0x80

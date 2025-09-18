.section .multiboot_header
.align 8

multiboot2_header_start:
    .long 0xe85250d6                # magic
    .long 0                         # architecture (i386)
    .long multiboot2_header_end - multiboot2_header_start  # header length
    
    # checksum
    .long -(0xe85250d6 + 0 + (multiboot2_header_end - multiboot2_header_start))
    
    # end tag
    .word 0    # type
    .word 0    # flags  
    .long 8    # size
multiboot2_header_end:
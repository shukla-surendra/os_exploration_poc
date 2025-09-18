#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::panic::PanicInfo;
use core::arch::{asm, global_asm};

// Multiboot2 header
global_asm!(
    r#"
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
    "#
);

mod vga;
mod idt;
mod pic;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Clear screen
    vga::clear_screen();
    vga::print_string("Rust OS - Interrupt Test from Scratch\n");
    vga::print_string("=====================================\n");

    // Setup IDT
    idt::init_idt();
    vga::print_string("IDT initialized\n");

    // Setup PIC
    pic::init_pic();
    vga::print_string("PIC initialized\n");

    // Enable interrupts
    unsafe {
        asm!("sti");
    }
    vga::print_string("Interrupts enabled\n");

    // Trigger a software interrupt for testing
    vga::print_string("Triggering test interrupt...\n");
    unsafe {
        asm!("int 0x80");
    }

    // Main loop
    vga::print_string("Entering main loop...\n");
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::print_string("KERNEL PANIC: ");
    if let Some(location) = info.location() {
        vga::print_string("at ");
        vga::print_string(location.file());
        vga::print_string(":");
        // Can't easily print numbers without std, so just show file
    }
    vga::print_string("\n");
    
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
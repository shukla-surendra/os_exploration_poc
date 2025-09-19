#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::{asm, global_asm};

// Multiboot2 constants for x86_64
const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;
const MB_ARCH_X86_64: u32 = 4; // architecture field (4 = x86_64 in multiboot2)
const FB_TAG_SIZE: u32 = 20;
const FB_TAG_PAD: u32 = (FB_TAG_SIZE + 7) & !7; // -> 24
const END_TAG_SIZE: u32 = 8;

// Total header length = 16 bytes for the 4 header words + framebuffer tag + end tag
const HEADER_LEN: u32 = 16 + FB_TAG_PAD + END_TAG_SIZE;

// checksum so (magic + arch + len + checksum) == 0 (u32 wrap)
const MB_CHECKSUM: u32 = (0u32).wrapping_sub(
    MULTIBOOT2_MAGIC.wrapping_add(MB_ARCH_X86_64).wrapping_add(HEADER_LEN)
);

// 8-byte alignment required by the spec
#[repr(C, align(8))]
struct MbHeader {
    pub magic: u32,
    pub arch: u32,
    pub len: u32,
    pub checksum: u32,

    pub fb_type: u16,
    pub fb_flags: u16,
    pub fb_size: u32,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_depth: u32,

    pub fb_pad: [u8; (FB_TAG_PAD - FB_TAG_SIZE) as usize], // 4 bytes

    pub end_type: u16,
    pub end_flags: u16,
    pub end_size: u32,
}

// Place header in a named section and keep it used so the linker does not discard it.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".multiboot2_header")]
#[used]
pub static MULTIBOOT2_HEADER: MbHeader = MbHeader {
    magic: MULTIBOOT2_MAGIC,
    arch: MB_ARCH_X86_64, // x86_64 architecture
    len: HEADER_LEN,
    checksum: MB_CHECKSUM,

    fb_type: 5,      // framebuffer tag type
    fb_flags: 0,
    fb_size: FB_TAG_SIZE,
    fb_width: 1024,
    fb_height: 768,
    fb_depth: 32,

    fb_pad: [0; 4],

    end_type: 0,
    end_flags: 0,
    end_size: END_TAG_SIZE,
};

mod vga;
mod idt;
mod pic;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Clear screen
    vga::clear_screen();
    vga::print_string("Rust OS - 64-bit Interrupt Test\n");
    vga::print_string("=================================\n");

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
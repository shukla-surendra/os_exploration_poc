#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Multiboot2 constants for x86_64
const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;
const MB_ARCH_X86_64: u32 = 4; // x86_64 architecture
const HEADER_LEN: u32 = 24;
const MB_CHECKSUM: u32 = 0u32.wrapping_sub(MULTIBOOT2_MAGIC.wrapping_add(MB_ARCH_X86_64).wrapping_add(HEADER_LEN));

#[repr(C, align(8))]
struct MbHeader {
    pub magic: u32,
    pub arch: u32,
    pub len: u32,
    pub checksum: u32,
    pub end_type: u16,
    pub end_flags: u16,
    pub end_size: u32,
}

#[unsafe(link_section = ".multiboot2")]
#[used]
pub static MULTIBOOT2_HEADER: MbHeader = MbHeader {
    magic: MULTIBOOT2_MAGIC,
    arch: MB_ARCH_X86_64,
    len: HEADER_LEN,
    checksum: MB_CHECKSUM,
    end_type: 0,
    end_flags: 0,
    end_size: 8,
};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop {} 
}
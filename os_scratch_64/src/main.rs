#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Multiboot2 constants
const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;
const MB_ARCH_I386:    u32 = 0; // architecture field (0 = i386 in multiboot2)
const MB_TAG_END_TYPE: u16 = 0;
const FB_TAG_SIZE: u32 = 20;
const FB_TAG_PAD: u32  = (FB_TAG_SIZE + 7) & !7; // -> 24
const MB_TAG_END_SIZE: u32 = 8; // end tag is 8 bytes (type(2)+flags(2)+size(4))
const END_TAG_SIZE: u32 = 8;

// Total header length = 16 bytes for the 4 header words + size of tags (we only have end tag = 8)
const HEADER_LEN: u32 = (16 + MB_TAG_END_SIZE) as u32;

// checksum so (magic + arch + len + checksum) == 0 (u32 wrap)
const MB_CHECKSUM: u32 = (0u32).wrapping_sub(MULTIBOOT2_MAGIC.wrapping_add(MB_ARCH_I386).wrapping_add(HEADER_LEN));

// 8-byte alignment required by the spec
#[repr(C, align(8))]
struct MbHeader {
    pub magic:    u32,
    pub arch:     u32,
    pub len:      u32,
    pub checksum: u32,

    pub fb_type:   u16,
    pub fb_flags:  u16,
    pub fb_size:   u32,
    pub fb_width:  u32,
    pub fb_height: u32,
    pub fb_depth:  u32,

    pub fb_pad:    [u8; (FB_TAG_PAD - FB_TAG_SIZE) as usize], // 4 bytes

    pub end_type:  u16,
    pub end_flags: u16,
    pub end_size:  u32,
}

// Place header in a named section and keep it used so the linker does not discard it.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".multiboot2")]
#[used]
pub static MULTIBOOT2_HEADER: MbHeader = MbHeader {
    magic: MULTIBOOT2_MAGIC,
    arch:  MB_ARCH_I386,
    len:   HEADER_LEN,
    checksum: MB_CHECKSUM,

    fb_type:  5,
    fb_flags: 0,
    fb_size:  FB_TAG_SIZE,
    fb_width: 1024,
    fb_height: 768,
    fb_depth: 32,

    fb_pad: [0; 4],

    end_type: 0,
    end_flags: 0,
    end_size: END_TAG_SIZE,
};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop {} 
}
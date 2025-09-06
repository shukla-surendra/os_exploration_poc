#![no_std]
#![no_main]


mod mem;
use core::panic::PanicInfo;
use core::arch::asm;
use core::mem::size_of;
use core::ptr::{read_unaligned, write_volatile};

#[repr(C)]
struct MbInfoHeader { total_size: u32, reserved: u32 }

#[repr(C)]
struct TagHeader { typ: u32, size: u32 }

#[repr(C)]
struct FramebufferInfo {
    typ: u32,
    size: u32,
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    // color info (variable) follows — not represented here
}

pub struct Framebuffer {
    pub phys_addr: usize,
    pub pitch: usize,
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
}

impl Framebuffer {

        /// Pack a 0xAARRGGBB color into the framebuffer format and write at (x,y).
    /// Supports common bpps: 32 (4 bytes), 24 (3 bytes), 16 (RGB565).
    /// 8bpp paletted is not handled here.
    pub unsafe fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height { return; }

        let base = self.phys_addr as *mut u8;
        let offset = y * self.pitch + x * (self.bpp / 8);
        let p = base.add(offset);

        match self.bpp {
            32 => {
                // write full u32
                let ptr = p as *mut u32;
                write_volatile(ptr, color);
            }
            24 => {
                // little-endian: write in memory as (B, G, R)
                // color is 0xAARRGGBB; extract bytes
                let b = (color & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let r = ((color >> 16) & 0xFF) as u8;
                // write bytes
                core::ptr::write_volatile(p, b);
                core::ptr::write_volatile(p.add(1), g);
                core::ptr::write_volatile(p.add(2), r);
            }
            16 => {
                // RGB565 packing: R:5 G:6 B:5
                let r8 = ((color >> 16) & 0xFF) as u16;
                let g8 = ((color >> 8) & 0xFF) as u16;
                let b8 = (color & 0xFF) as u16;
                let r5 = (r8 >> 3) & 0x1F;
                let g6 = (g8 >> 2) & 0x3F;
                let b5 = (b8 >> 3) & 0x1F;
                let pixel16: u16 = (r5 << 11) | (g6 << 5) | b5;
                let ptr16 = p as *mut u16;
                write_volatile(ptr16, pixel16);
            }
            other => {
                // unsupported bpp: do nothing or fallback
                let _ = other;
            }
        }
    }

    /// Fast horizontal fill of a row for bytes-per-pixel that is a power of two.
    /// Used to implement fast clear/rect.
    unsafe fn fill_row_bytes(&self, y: usize, x0: usize, x1: usize, pixel_bytes: &[u8]) {
        if y >= self.height || x0 >= x1 { return; }
        let base = self.phys_addr as *mut u8;
        let stride = self.pitch;
        let start = base.add(y * stride + x0 * pixel_bytes.len());
        let mut dst = start;
        let count = x1 - x0;
        // naive loop; can be optimized w/ word writes or memcpy-like writes
        for _ in 0..count {
            // write bytes of pixel
            for i in 0..pixel_bytes.len() {
                core::ptr::write_volatile(dst.add(i), pixel_bytes[i]);
            }
            dst = dst.add(pixel_bytes.len());
        }
    }

    /// Draw filled rectangle. Uses put_pixel currently; for speed use fill_row_bytes.
    pub unsafe fn fill_rect(&self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let x1 = (x + w).min(self.width);
        let y1 = (y + h).min(self.height);
        if self.bpp == 32 {
            for yy in y..y1 {
                let base = self.phys_addr as *mut u8;
                let mut ptr = base.add(yy * self.pitch + x * 4) as *mut u32;
                for _ in x..x1 {
                    write_volatile(ptr, color);
                    ptr = ptr.add(1);
                }
            }
        } else {
            for yy in y..y1 {
                for xx in x..x1 {
                    self.put_pixel(xx, yy, color);
                }
            }
        }
    }

    /// Bresenham line (integer) — draws a 1px wide line.
    pub unsafe fn draw_line(&self, x0: isize, y0: isize, x1: isize, y1: isize, color: u32) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        loop {
            if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
                self.put_pixel(x as usize, y as usize, color);
            }
            if x == x1 && y == y1 { break; }
            let e2 = 2*err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw a simple gradient background (horizontal).
    pub unsafe fn draw_gradient(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                // mix two colors based on x/width
                let t = (x * 255) / (self.width.saturating_sub(1));
                // red to blue gradient
                let r = t as u32;
                let g = ((y * 128) / (self.height.saturating_sub(1))) as u32;
                let b = (255 - t) as u32;
                let color = (0xFF << 24) | (r << 16) | (g << 8) | b;
                self.put_pixel(x, y, color);
            }
        }
    }
    /// Write a pixel in 32bpp (ARGB/ABGR layout depends on platform).
    /// color is 0xAARRGGBB (alpha ignored for many modes).
    /// This assumes physical == virtual (identity mapping). Map if using paging.
    pub unsafe fn put_pixel_32(&self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height { return; }
        let base = self.phys_addr as *mut u8;
        let offset = y * self.pitch + x * 4;
        let ptr = base.add(offset) as *mut u32;
        write_volatile(ptr, color);
    }

    /// Clear screen (32bpp) to color.
    pub unsafe fn clear_32(&self, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.put_pixel_32(x, y, color);
            }
        }
    }
}


/// Parse the Multiboot2 info block (mbi_ptr from EBX) and return Framebuffer if available.
///
/// Safety: Caller must ensure mbi_ptr is a valid pointer (provided by bootloader) and
/// that physical addresses are accessible (identity mapped) if you dereference framebuffer.
unsafe fn find_framebuffer(mbi_ptr: u32) -> Option<Framebuffer> {
    let base = mbi_ptr as *const u8;

    // Read total_size first (mbi header)
    let total_size = read_unaligned(base as *const u32) as usize;
    if total_size == 0 { return None; }

    // Tags start at offset 8
    let mut offset: usize = 8;
    while offset + core::mem::size_of::<TagHeader>() <= total_size {
        let tag_ptr = base.add(offset) as *const TagHeader;
        // read unaligned tag header
        let tag = read_unaligned(tag_ptr);
        if tag.typ == 0 && tag.size == 8 {
            // end tag
            break;
        }

        if tag.typ == 8 {
            // framebuffer info tag found
            // ensure we can read framebuffer info struct fully
            if offset + (tag.size as usize) > total_size {
                return None;
            }
            let fb_ptr = base.add(offset) as *const FramebufferInfo;
            let fb = read_unaligned(fb_ptr);

            let addr = fb.framebuffer_addr as usize;
            let pitch = fb.framebuffer_pitch as usize;
            let width = fb.framebuffer_width as usize;
            let height = fb.framebuffer_height as usize;
            let bpp = fb.framebuffer_bpp as usize;

            return Some(Framebuffer { phys_addr: addr, pitch, width, height, bpp });
        }

        // advance to next tag, tags are padded to 8 bytes
        let mut next = offset + ((tag.size as usize + 7) & !7usize);
        if next <= offset { break; } // sanity
        offset = next;
    }

    None
}


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
    let magic: u32;
    let info_ptr: u32;

    // Read eax and ebx directly
    unsafe {
        asm!(
            "mov {0:e}, eax",
            "mov {1:e}, ebx",
            out(reg) magic,
            out(reg) info_ptr,
            options(nostack)
        );
    }

    if magic != 0x36d76289 {
        loop {}
    }


       let fb_opt = unsafe { find_framebuffer(info_ptr) };

if let Some(fb) = fb_opt {
    unsafe {
        if fb.bpp == 32 {
            // gradient fills the whole screen (visual test)
            fb.draw_gradient();

            // draw border rectangle
            fb.fill_rect(20, 20, fb.width - 40, fb.height - 40, 0xFF_00_80_00);

            // draw diagonal lines
            fb.draw_line(0, 0, (fb.width-1) as isize, (fb.height-1) as isize, 0xFF_FF_00_00);
            fb.draw_line((fb.width-1) as isize, 0, 0, (fb.height-1) as isize, 0xFF_00_FF_00);
        } else {
            // fallback: paint a solid color
            fb.clear_32(0xFF_20_20_40);
        }
    }
}

    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }

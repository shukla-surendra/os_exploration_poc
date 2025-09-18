use core::ptr;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

pub fn clear_screen() {
    unsafe {
        for i in 0..(VGA_WIDTH * VGA_HEIGHT * 2) {
            ptr::write_volatile(VGA_BUFFER.add(i), if i % 2 == 0 { b' ' } else { 0x07 });
        }
        CURSOR_X = 0;
        CURSOR_Y = 0;
    }
}

pub fn put_char(c: u8, color: u8) {
    unsafe {
        if c == b'\n' {
            CURSOR_X = 0;
            CURSOR_Y += 1;
            if CURSOR_Y >= VGA_HEIGHT {
                scroll_up();
                CURSOR_Y = VGA_HEIGHT - 1;
            }
            return;
        }

        let offset = (CURSOR_Y * VGA_WIDTH + CURSOR_X) * 2;
        ptr::write_volatile(VGA_BUFFER.add(offset), c);
        ptr::write_volatile(VGA_BUFFER.add(offset + 1), color);

        CURSOR_X += 1;
        if CURSOR_X >= VGA_WIDTH {
            CURSOR_X = 0;
            CURSOR_Y += 1;
            if CURSOR_Y >= VGA_HEIGHT {
                scroll_up();
                CURSOR_Y = VGA_HEIGHT - 1;
            }
        }
    }
}

pub fn print_string(s: &str) {
    for byte in s.bytes() {
        put_char(byte, 0x07); // White on black
    }
}

pub fn print_hex(value: u64) {
    let hex_chars = b"0123456789ABCDEF";
    print_string("0x");
    for i in (0..16).rev() {
        let nibble = ((value >> (i * 4)) & 0xF) as usize;
        put_char(hex_chars[nibble], 0x0A); // Light green
    }
}

fn scroll_up() {
    unsafe {
        // Move all lines up by one
        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let src_offset = (y * VGA_WIDTH + x) * 2;
                let dst_offset = ((y - 1) * VGA_WIDTH + x) * 2;
                
                let ch = ptr::read_volatile(VGA_BUFFER.add(src_offset));
                let color = ptr::read_volatile(VGA_BUFFER.add(src_offset + 1));
                
                ptr::write_volatile(VGA_BUFFER.add(dst_offset), ch);
                ptr::write_volatile(VGA_BUFFER.add(dst_offset + 1), color);
            }
        }
        
        // Clear the last line
        for x in 0..VGA_WIDTH {
            let offset = ((VGA_HEIGHT - 1) * VGA_WIDTH + x) * 2;
            ptr::write_volatile(VGA_BUFFER.add(offset), b' ');
            ptr::write_volatile(VGA_BUFFER.add(offset + 1), 0x07);
        }
    }
}
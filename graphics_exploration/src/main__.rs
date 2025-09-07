use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1366;
const HEIGHT: usize = 786;

// A tiny 8x8 font for characters '0'..'9' and space — expand as needed.
const FONT8X8_DIGITS: [[u8; 8]; 11] = [
    // '0'
    [0x3C,0x42,0x46,0x4A,0x52,0x62,0x42,0x3C],
    // '1'
    [0x08,0x18,0x28,0x08,0x08,0x08,0x08,0x3E],
    // '2'
    [0x3C,0x42,0x02,0x04,0x08,0x10,0x20,0x7E],
    // '3'
    [0x3C,0x42,0x02,0x1C,0x02,0x02,0x42,0x3C],
    // '4'
    [0x04,0x0C,0x14,0x24,0x44,0x7E,0x04,0x04],
    // '5'
    [0x7E,0x40,0x40,0x7C,0x02,0x02,0x42,0x3C],
    // '6'
    [0x1C,0x20,0x40,0x7C,0x42,0x42,0x42,0x3C],
    // '7'
    [0x7E,0x02,0x04,0x08,0x10,0x10,0x10,0x10],
    // '8'
    [0x3C,0x42,0x42,0x3C,0x42,0x42,0x42,0x3C],
    // '9'
    [0x3C,0x42,0x42,0x42,0x3E,0x02,0x04,0x38],
    // ' ' (space)
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
];

fn put_pixel(buffer: &mut [u32], x: isize, y: isize, color: u32) {
    if x < 0 || y < 0 { return; }
    let x = x as usize;
    let y = y as usize;
    if x >= WIDTH || y >= HEIGHT { return; }
    buffer[y * WIDTH + x] = color;
}

fn draw_char_8x8(buffer: &mut [u32], ch: char, x: isize, y: isize, color: u32) {
    let idx = match ch {
        '0'..='9' => (ch as u8 - b'0') as usize,
        ' ' => 10,
        _ => 10, // fallback to space for unsupported chars
    };
    let glyph = &FONT8X8_DIGITS[idx];
    for row in 0..8 {
        let bits = glyph[row];
        for col in 0..8 {
            if (bits >> (7 - col)) & 1 == 1 {
                put_pixel(buffer, x + col as isize, y + row as isize, color);
            }
        }
    }
}

fn draw_text_8x8(buffer: &mut [u32], text: &str, x: isize, y: isize, color: u32) {
    let mut ox = x;
    for ch in text.chars() {
        draw_char_8x8(buffer, ch, ox, y, color);
        ox += 8; // move by 8 pixels per char
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new("Text - ESC to exit", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.set_target_fps(60);

    let mut t = 0u32;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // clear black
        for p in buffer.iter_mut() { *p = 0xFF000000; }

        // draw some text
        draw_text_8x8(&mut buffer, "0123456789", 20, 20, 0xFFFFFFFF);
        draw_text_8x8(&mut buffer, "score:", 20, 40, 0xFFFFFF00);

        // moving number
        draw_text_8x8(&mut buffer, &format!("{}", t % 100), 100, 40, 0xFF00FF00);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        t += 1;
    }
}

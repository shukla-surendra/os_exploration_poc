use minifb::{Key, Window, WindowOptions};
use rusttype::{Font, Scale, point};

const WIDTH: usize = 1366;
const HEIGHT: usize = 786;

fn put_pixel(buffer: &mut [u32], x: isize, y: isize, color: u32) {
    if x < 0 || y < 0 { return; }
    let x = x as usize;
    let y = y as usize;
    if x >= WIDTH || y >= HEIGHT { return; }
    buffer[y * WIDTH + x] = color;
}

// simple alpha blend src_color (ARGB with alpha in top 8 bits) over dest (u32)
fn blend_pixel(buffer: &mut [u32], x: isize, y: isize, src_color: u32) {
    if x < 0 || y < 0 { return; }
    let x = x as usize;
    let y = y as usize;
    if x >= WIDTH || y >= HEIGHT { return; }

    let dst_idx = y * WIDTH + x;
    let dst = buffer[dst_idx];

    // extract components
    let sa = ((src_color >> 24) & 0xFF) as f32 / 255.0;
    let sr = ((src_color >> 16) & 0xFF) as f32;
    let sg = ((src_color >> 8) & 0xFF) as f32;
    let sb = (src_color & 0xFF) as f32;

    let da = ((dst >> 24) & 0xFF) as f32 / 255.0;
    let dr = ((dst >> 16) & 0xFF) as f32;
    let dg = ((dst >> 8) & 0xFF) as f32;
    let db = (dst & 0xFF) as f32;

    // premultiplied-like blend: out = src*sa + dst*(1-sa)
    let out_r = (sr * sa + dr * (1.0 - sa)).round().clamp(0.0, 255.0) as u32;
    let out_g = (sg * sa + dg * (1.0 - sa)).round().clamp(0.0, 255.0) as u32;
    let out_b = (sb * sa + db * (1.0 - sa)).round().clamp(0.0, 255.0) as u32;
    let out_a = ((sa + da * (1.0 - sa)) * 255.0).round().clamp(0.0, 255.0) as u32;

    buffer[dst_idx] = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
}

fn draw_text_rusttype(buffer: &mut [u32], font: &Font, text: &str, x: f32, y: f32, scale: f32, color: u32) {
    let scale = Scale::uniform(scale);
    // baseline point: rusttype positions glyphs relative to baseline.
    let v_metrics = font.v_metrics(scale);
    let start = point(x, y + v_metrics.ascent);

    for glyph in font.layout(text, scale, start) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            // draw the glyph: rusttype provides coverage [0.0..1.0] as 'v' in the closure
            glyph.draw(|gx, gy, v| {
                let px = gx as i32 + bb.min.x;
                let py = gy as i32 + bb.min.y;
                if px >= 0 && py >= 0 && (px as usize) < WIDTH && (py as usize) < HEIGHT {
                    // create src_color with alpha = v
                    let alpha = (v * 255.0).round() as u32;
                    let src_color = (alpha << 24) | ( ( (color >> 16) & 0xFF) << 16 ) | ( ( (color >> 8) & 0xFF) << 8 ) | (color & 0xFF);
                    blend_pixel(buffer, px as isize, py as isize, src_color);
                }
            });
        }
    }
}

fn main() {
    let font_data = include_bytes!("../fonts/DejaVuSans.ttf") as &[u8]; // put a ttf next to src
    let font = Font::try_from_bytes(font_data).expect("Error constructing Font");

    let mut buffer: Vec<u32> = vec![0xFF000000; WIDTH * HEIGHT]; // opaque black
    let mut window = Window::new("Text (rusttype) - ESC to exit", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.set_target_fps(60);

    let mut t = 0u32;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // simple clear
        for p in buffer.iter_mut() { *p = 0xFF000000; }

        draw_text_rusttype(&mut buffer, &font, "This is Rust Program ! Using Rusttype for Text Rendering", 20.0, 50.0, 32.0, 0x00FF_FFFF); // cyan-ish (RRGGBB)
        draw_text_rusttype(&mut buffer, &font, &format!("Frame: {}", t), 20.0, 100.0, 20.0, 0xFF00_FF00); // green
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        t += 1;
    }
}

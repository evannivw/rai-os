//! Rasterized Ubuntu Light ASCII glyphs. No font engine or allocation in the guest.
use super::{Display, glyphs::GLYPHS};
pub struct Glyph {
    pub offset: usize,
    pub width: usize,
    pub height: usize,
    pub left: i32,
    pub top: i32,
    pub advance: i32,
}
static COVERAGE: &[u8] = include_bytes!("../../assets/fonts/rai-ascii.bin");
#[derive(Clone, Copy)]
pub enum Size {
    Small,
    Body,
    Title,
}
fn glyph(byte: u8, size: Size) -> &'static Glyph {
    let byte = if (32..=126).contains(&byte) {
        byte
    } else {
        b'?'
    };
    &GLYPHS[size as usize * 95 + (byte - 32) as usize]
}
pub fn width(text: &str, size: Size) -> i32 {
    text.bytes().map(|b| glyph(b, size).advance).sum()
}
pub fn draw(surface: &mut Display, text: &str, mut x: i32, y: i32, size: Size, color: u32) {
    for byte in text.bytes() {
        let g = glyph(byte, size);
        for dy in 0..g.height {
            for dx in 0..g.width {
                let pixel = g.offset + dy * g.width + dx;
                let packed = COVERAGE[pixel / 2];
                let alpha = (if pixel.is_multiple_of(2) {
                    packed >> 4
                } else {
                    packed & 15
                }) * 17;
                if alpha != 0 {
                    surface.blend(
                        x + g.left + dx as i32,
                        y + g.top + dy as i32,
                        color,
                        u32::from(alpha),
                    );
                }
            }
        }
        x += g.advance;
    }
}

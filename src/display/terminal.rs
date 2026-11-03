//! Shared typography and geometry. Every pixel is drawn by the guest.
use super::{
    Display,
    font::{self, Size},
};
pub const BG: u32 = 0x080a0d;
pub const PANEL: u32 = 0x12171d;
pub const INK: u32 = 0xe4e9ef;
pub const MUTED: u32 = 0x85919f;
pub const ACCENT: u32 = 0xc1f27d;
pub const LINE: u32 = 0x27313c;
pub fn rect(s: &mut Display, x: usize, y: usize, w: usize, h: usize, c: u32) {
    for dy in y..y + h {
        for dx in x..x + w {
            s.pixel(dx, dy, c);
        }
    }
}
pub fn text(s: &mut Display, t: &str, x: i32, y: i32, c: u32) {
    font::draw(s, t, x, y, Size::Small, c);
}
pub fn body(s: &mut Display, t: &str, x: i32, y: i32, c: u32) {
    font::draw(s, t, x, y, Size::Body, c);
}
pub fn wrap(s: &mut Display, t: &str, y: i32, rows: usize, c: u32) {
    for (i, line) in t.as_bytes().chunks(56).take(rows).enumerate() {
        text(
            s,
            core::str::from_utf8(line).unwrap_or("?"),
            30,
            y + i as i32 * 20,
            c,
        );
    }
}
pub fn base(s: &mut Display, section: &str, title: &str, subtitle: &str, live: bool) {
    s.fill(BG);
    rect(s, 24, 27, 9, 9, ACCENT);
    text(s, "RAI / OS", 44, 23, INK);
    text(
        s,
        if live {
            "META / PLANNER"
        } else {
            "REPLAY / OFFLINE"
        },
        330,
        23,
        ACCENT,
    );
    rect(s, 24, 58, 432, 1, LINE);
    text(s, section, 26, 78, MUTED);
    font::draw(s, title, 24, 102, Size::Title, INK);
    text(s, subtitle, 26, 150, MUTED);
}
pub fn button(s: &mut Display, label: &str, footer: &str) {
    rect(s, 24, 706, 432, 48, PANEL);
    rect(s, 24, 706, 3, 48, ACCENT);
    body(s, label, 42, 718, ACCENT);
    text(s, footer, 24, 775, MUTED);
}
pub fn prompt(s: &mut Display, label: &str, value: &str) {
    text(s, label, 26, 552, ACCENT);
    rect(s, 24, 580, 432, 108, PANEL);
    wrap(s, value, 593, 4, INK);
}
pub fn divider(s: &mut Display, y: usize) {
    rect(s, 24, y, 432, 1, LINE);
}

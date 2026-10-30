//! Pure presentation of observed outcomes. No device registers, policy or AI calls.
use super::{
    Display,
    font::{self, Size},
};
use crate::records::RecordId;
use core::fmt::{self, Write};
const PAPER: u32 = 0xf6f7f2;
const INK: u32 = 0x203b34;
const MUTED: u32 = 0x657970;
const GREEN: u32 = 0x367763;
const LINE: u32 = 0xdce3da;
#[derive(Clone, Copy)]
pub enum View<'a> {
    Ready,
    CoordinatorReady,
    Index(RecordId, &'a str),
    Reused(RecordId, &'a str),
    Pending(RecordId, u64),
    Error(&'a str),
}
struct Label {
    bytes: [u8; 64],
    len: usize,
}
impl Label {
    fn new() -> Self {
        Self {
            bytes: [0; 64],
            len: 0,
        }
    }
    fn text(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}
impl fmt::Write for Label {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > self.bytes.len() - self.len {
            return Err(fmt::Error);
        }
        self.bytes[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}
fn text(s: &mut Display, value: &str, x: i32, y: i32, size: Size, color: u32) {
    font::draw(s, value, x, y, size, color);
}
fn circle(s: &mut Display, cx: i32, cy: i32, radius: i32, color: u32) {
    for y in -radius - 1..=radius + 1 {
        for x in -radius - 1..=radius + 1 {
            let coverage = ((radius + 1) * (radius + 1) - x * x - y * y) * 255 / (2 * radius + 1);
            if coverage > 0 {
                s.blend(cx + x, cy + y, color, coverage.min(255) as u32);
            }
        }
    }
}
fn line(s: &mut Display, y: usize) {
    for x in 40..440 {
        s.pixel(x, y, LINE);
    }
}
/// Decorative identity: three fine tilted ellipses, not fabricated live telemetry.
fn constellation(s: &mut Display) {
    for y in 275..508_i32 {
        for x in 64..424_i32 {
            let dx = x - 244;
            let dy = y - 390;
            let u = i64::from(dx * 866 - dy * 500);
            let v = i64::from(dx * 500 + dy * 866);
            for (rx, ry, color) in [(149, 69, 0xb9d3be), (121, 46, 0x83af98), (91, 25, 0x558c76)] {
                let q =
                    u * u * 10000 / (rx * rx * 1_000_000) + v * v * 10000 / (ry * ry * 1_000_000);
                let delta = (q - 10000).abs();
                if delta < 320 {
                    s.blend(x, y, color, ((320 - delta) * 180 / 320) as u32);
                }
            }
        }
    }
    circle(s, 244, 390, 27, 0xe3eddf);
    circle(s, 244, 390, 14, INK);
    circle(s, 247, 386, 3, 0xd3e7bb);
    circle(s, 134, 453, 6, GREEN);
    circle(s, 366, 325, 4, 0x91b49a);
    for x in 93..104 {
        s.pixel(x, 326, 0x9bb5a0);
    }
    for y in 321..332 {
        s.pixel(98, y, 0x9bb5a0);
    }
    text(
        s,
        "A little space. A new possibility.",
        138,
        494,
        Size::Small,
        MUTED,
    );
}
pub fn render(s: &mut Display, view: View<'_>) {
    s.fill(PAPER);
    // Small original mark: two seeds, intentionally separate from any app icon grid.
    circle(s, 47, 39, 6, INK);
    circle(s, 59, 31, 3, GREEN);
    text(s, "rai", 73, 22, Size::Body, INK);
    text(s, "OS", 104, 27, Size::Small, MUTED);
    text(s, "EARLY PREVIEW", 336, 28, Size::Small, MUTED);
    text(s, "Room to", 40, 91, Size::Title, INK);
    text(s, "think.", 40, 143, Size::Title, INK);
    text(s, "An operating system,", 43, 217, Size::Body, MUTED);
    text(s, "with intention.", 43, 243, Size::Body, MUTED);
    constellation(s);
    line(s, 538);
    text(s, "01 / CONTENT", 40, 554, Size::Small, MUTED);
    match view {
        View::Ready | View::CoordinatorReady => {
            circle(s, 430, 563, 4, GREEN);
            text(s, "Make sense of a note.", 40, 580, Size::Body, INK);
            text(
                s,
                "Your first content index starts here.",
                40,
                618,
                Size::Small,
                MUTED,
            );
            text(s, "In the console, enter:", 40, 655, Size::Small, MUTED);
            text(
                s,
                if matches!(view, View::CoordinatorReady) {
                    "coord.observe notes@1"
                } else {
                    "record.index notes@1"
                },
                40,
                680,
                Size::Body,
                GREEN,
            );
        }
        View::Index(id, output) | View::Reused(id, output) => {
            let mut label = Label::new();
            let _ = write!(label, "{id}");
            text(s, label.text(), 40, 580, Size::Body, INK);
            text(
                s,
                if matches!(view, View::Reused(..)) {
                    "INDEX REUSED"
                } else {
                    "INDEX CREATED"
                },
                336,
                558,
                Size::Small,
                GREEN,
            );
            if output == "-" {
                text(
                    s,
                    "An empty note. Nothing to index yet.",
                    40,
                    621,
                    Size::Small,
                    MUTED,
                );
            } else {
                let mut label = Label::new();
                let distinct = output.split(',').count();
                if let Some(total) = super::summary::token_total(output) {
                    let _ = write!(
                        label,
                        "{total} words  /  {distinct} distinct  /  {} bytes",
                        id.bytes()
                    );
                } else {
                    let _ = write!(label, "Index statistics unavailable");
                }
                text(s, label.text(), 40, 613, Size::Small, MUTED);
                let mut x = 40;
                let mut y = 645;
                for token in output.split(',') {
                    let (word, count) = token.split_once(':').unwrap_or((token, "?"));
                    let mut label = Label::new();
                    if write!(label, "{word}  {count}").is_err() {
                        text(s, "More in console", 40, y, Size::Small, MUTED);
                        break;
                    }
                    let width = font::width(label.text(), Size::Small) + 22;
                    if x + width > 444 {
                        x = 40;
                        y += 25;
                    }
                    if y > 720 {
                        text(s, "More in console", 40, 720, Size::Small, MUTED);
                        break;
                    }
                    text(s, label.text(), x, y, Size::Small, INK);
                    x += width;
                }
            }
        }
        View::Pending(id, ticket) => {
            text(s, "WAITING FOR PLAN", 309, 558, Size::Small, MUTED);
            let mut label = Label::new();
            let _ = write!(label, "{id}");
            text(s, label.text(), 40, 580, Size::Body, INK);
            text(
                s,
                "The coordinator is waiting for a proposal.",
                40,
                621,
                Size::Small,
                MUTED,
            );
            let mut label = Label::new();
            let _ = write!(label, "coord.propose {ticket}:{id}");
            text(s, label.text(), 40, 667, Size::Small, GREEN);
        }
        View::Error(code) => {
            text(s, "NOT COMPLETED", 323, 558, Size::Small, 0x98644c);
            text(s, "That request needs a look.", 40, 580, Size::Body, INK);
            text(s, code, 40, 621, Size::Small, 0x98644c);
            text(
                s,
                "No new index was published.",
                40,
                659,
                Size::Small,
                MUTED,
            );
        }
    }
    line(s, 751);
    text(s, "RAI-OS", 40, 765, Size::Small, INK);
    text(s, "Native display / QEMU", 306, 765, Size::Small, MUTED);
    s.present();
}

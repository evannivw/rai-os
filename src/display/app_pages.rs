//! Guest-rendered data pages. Geometry is fixed; models supply bounded content.
use super::{
    Display,
    font::{self, Size},
    terminal as t,
};
use crate::{
    app_ui::{Controller, Mode},
    microapp::{App, Block, pages::BlockKind},
};
const INK: u32 = 0xe7ebf1;
const QUIET: u32 = 0xa1adbc;
const ACCENT: u32 = 0x87bfff;
fn text(d: &mut Display, s: &str, x: i32, y: i32, c: u32) {
    t::text(d, s, x, y, c);
}
fn clipped(s: &str, n: usize) -> &str {
    &s[..s.len().min(n)]
}
fn number(d: &mut Display, n: i64, x: i32, y: i32, large: bool) {
    let mut buf = [0u8; 24];
    let mut i = buf.len();
    let mut v = n.unsigned_abs();
    loop {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    if n < 0 {
        i -= 1;
        buf[i] = b'-';
    }
    let s = core::str::from_utf8(&buf[i..]).unwrap_or("?");
    font::draw(
        d,
        s,
        x,
        y,
        if large { Size::Title } else { Size::Small },
        ACCENT,
    );
}
fn content(d: &mut Display, a: &App, b: &Block, y: i32) {
    text(d, b.title.text(), 32, y, INK);
    text(d, b.scope(), 390, y, QUIET);
    match b.kind {
        BlockKind::Text => {
            let value = b.value.text();
            let split = value.len().min(54);
            text(d, &value[..split], 32, y + 38, INK);
            if split < value.len() {
                text(d, &value[split..], 32, y + 60, INK);
            }
        }
        BlockKind::Metric => {
            text(d, b.aggregate.name(), 32, y + 35, QUIET);
            if let Some(n) = b.metric(a) {
                number(d, n, 32, y + 76, true);
            } else {
                text(d, "No numeric values", 32, y + 80, QUIET);
            }
        }
        BlockKind::Bars | BlockKind::Table => {
            let Some(value) = b.value_index(&a.spec) else {
                return;
            };
            let Some(label) = b.label_index(&a.spec) else {
                return;
            };
            let mut count = 0;
            let mut max_abs = 1i64;
            for row in 0..a.count() {
                if b.visible(a, row)
                    && let Some(n) = a.row(row).and_then(|r| r[value].text().parse::<i32>().ok())
                {
                    max_abs = max_abs.max(i64::from(n).abs());
                }
            }
            if b.kind == BlockKind::Table {
                text(
                    d,
                    clipped(a.spec.fields()[label].label.text(), 24),
                    32,
                    y + 28,
                    QUIET,
                );
                text(
                    d,
                    clipped(a.spec.fields()[value].label.text(), 24),
                    252,
                    y + 28,
                    QUIET,
                );
            } else {
                text(d, "label / value", 32, y + 28, QUIET);
                text(d, "-  0  +", 338, y + 28, QUIET);
                t::rect(d, 366, (y + 52) as usize, 1, 160, 0x465366);
            }
            for row in 0..a.count() {
                if !b.visible(a, row) {
                    continue;
                }
                let values = a.row(row).unwrap();
                let line = y + 54 + count * 20;
                count += 1;
                text(
                    d,
                    clipped(
                        values[label].text(),
                        if b.kind == BlockKind::Table { 26 } else { 16 },
                    ),
                    32,
                    line,
                    INK,
                );
                if b.kind == BlockKind::Table {
                    text(d, clipped(values[value].text(), 26), 252, line, INK);
                } else if let Ok(n) = values[value].text().parse::<i32>() {
                    let n = i64::from(n);
                    number(d, n, 160, line, false);
                    let width = (n.abs() * 64 / max_abs) as usize;
                    if width > 0 {
                        t::rect(
                            d,
                            if n < 0 { 366 - width } else { 367 },
                            (line + 3) as usize,
                            width,
                            9,
                            ACCENT,
                        );
                    }
                } else {
                    text(d, "empty", 160, line, QUIET);
                }
            }
            if count == 0 {
                text(d, "No matching records", 32, y + 65, QUIET);
            }
        }
    }
    t::rect(d, 32, (y + 219) as usize, 416, 1, 0x2a3543);
}
pub fn render(d: &mut Display, u: &Controller) {
    let review = u.mode == Mode::PageReview;
    let spec = if review {
        &u.pending
    } else {
        let Some(a) = u.runtime.app(u.selected) else {
            return;
        };
        &a.spec
    };
    let Some(page) = spec.pages.entries().get(u.page) else {
        return;
    };
    font::draw(d, page.title.text(), 32, 128, Size::Body, ACCENT);
    if review {
        for (i, b) in page.blocks().iter().enumerate() {
            let y = 190 + i as i32 * 220;
            text(d, b.kind.name(), 32, y, QUIET);
            text(d, b.title.text(), 32, y + 28, INK);
            let value = b.value.text();
            let n = value.len().min(54);
            text(d, &value[..n], 32, y + 60, INK);
            if n < value.len() {
                text(d, &value[n..], 32, y + 80, INK);
            }
            text(d, "label field / aggregate / scope", 32, y + 113, QUIET);
            text(d, b.label_field.text(), 32, y + 138, INK);
            text(d, b.aggregate.name(), 174, y + 138, INK);
            text(d, b.scope(), 300, y + 138, ACCENT);
            t::rect(d, 32, (y + 189) as usize, 416, 1, 0x2a3543);
        }
        text(d, "Read-only pages. No data is changed.", 32, 643, QUIET);
    } else {
        let a = u.runtime.app(u.selected).unwrap();
        for (i, b) in page.blocks().iter().enumerate() {
            content(d, a, b, 190 + i as i32 * 230);
        }
        t::rect(d, 24, 696, 210, 48, 0x111923);
        t::rect(d, 246, 696, 210, 48, 0x111923);
        text(
            d,
            "< Previous",
            46,
            710,
            if u.page == 0 { QUIET } else { ACCENT },
        );
        text(
            d,
            "Next >",
            330,
            710,
            if u.page + 1 == spec.pages.entries().len() {
                QUIET
            } else {
                ACCENT
            },
        );
    }
}

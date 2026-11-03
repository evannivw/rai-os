//! Phone-shaped launcher and app views. Geometry matches guest-owned hit testing.
use super::{
    Display,
    font::{self, Size},
    terminal as t,
};
use crate::{
    app_ui::{Controller, Mode},
    microapp::{Capability, Kind},
};
const QUIET: u32 = 0x62716d;
const WHITE: u32 = 0xe6ece7;
const LIME: u32 = 0xc1f27d;
fn text(s: &mut Display, v: &str, x: i32, y: i32, c: u32) {
    t::text(s, v, x, y, c);
}
fn center(s: &mut Display, v: &str, y: i32, size: Size, c: u32) {
    font::draw(s, v, (480 - font::width(v, size)) / 2, y, size, c);
}
/// Broken horizontal strokes, fine verticals and diagonal corners echo terminal
/// line art without depending on Unicode font support or a host renderer.
fn frame(s: &mut Display, x: usize, y: usize, w: usize, h: usize, c: u32) {
    for dx in 9..w - 9 {
        if dx % 12 < 9 {
            s.pixel(x + dx, y, c);
            s.pixel(x + dx, y + h - 1, c);
        }
    }
    for dy in 9..h - 9 {
        s.pixel(x, y + dy, c);
        s.pixel(x + w - 1, y + dy, c);
    }
    for d in 0..9 {
        s.pixel(x + 8 - d, y + d, c);
        s.pixel(x + w - 9 + d, y + d, c);
        s.pixel(x + 8 - d, y + h - 1 - d, c);
        s.pixel(x + w - 9 + d, y + h - 1 - d, c);
    }
}
fn icon(s: &mut Display, x: usize, y: usize, shape: usize, c: u32) {
    frame(s, x, y, 70, 70, c);
    let art = match shape % 4 {
        0 => ["+---+", "| = |", "+---+"],
        1 => ["[x] /", "[ ] /", "[ ] /"],
        2 => ["+---+", "|# #|", "+---+"],
        _ => ["  |  ", "--+--", "  |  "],
    };
    for (i, line) in art.iter().enumerate() {
        text(s, line, x as i32 + 18, y as i32 + 10 + i as i32 * 17, c);
    }
}
fn header(s: &mut Display, u: &Controller, title: &str) {
    s.fill(0x060908);
    text(s, "rai", 26, 28, WHITE);
    text(s, if u.live { "meta" } else { "replay" }, 406, 28, QUIET);
    center(s, ". . .", 28, Size::Small, QUIET);
    center(s, title, 83, Size::Title, WHITE);
}
fn nav(s: &mut Display) {
    t::rect(s, 24, 750, 432, 1, 0x21302a);
    text(s, "[ home ]", 49, 769, QUIET);
    text(s, "[ + create ]", 198, 769, LIME);
    text(s, "[ / find ]", 365, 769, QUIET);
}
fn action(s: &mut Display, label: &str) {
    frame(s, 24, 696, 432, 48, LIME);
    center(s, label, 708, Size::Body, LIME);
}
fn wrap(s: &mut Display, value: &str, y: i32, lines: usize) {
    let mut rest = value;
    for row in 0..lines {
        if rest.is_empty() {
            break;
        }
        let mut end = rest.len().min(54);
        if end < rest.len()
            && let Some(space) = rest[..end].rfind(' ').filter(|i| *i > 0)
        {
            end = space;
        }
        text(s, &rest[..end], 42, y + row as i32 * 21, WHITE);
        rest = rest[end..].trim_start();
    }
}
fn number(s: &mut Display, value: i64, x: i32, y: i32) {
    let mut bytes = [0u8; 24];
    let mut end = 24;
    let mut n = value.unsigned_abs();
    loop {
        end -= 1;
        bytes[end] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if value < 0 {
        end -= 1;
        bytes[end] = b'-';
    }
    text(
        s,
        core::str::from_utf8(&bytes[end..]).unwrap_or("?"),
        x,
        y,
        LIME,
    );
}
pub fn render(s: &mut Display, u: &Controller) {
    let a = u.runtime.app(u.selected);
    let title = match u.mode {
        Mode::Home => "Your space.",
        Mode::Search => "Find an app.",
        Mode::Compose => {
            if u.target().is_some() {
                "Evolve your app."
            } else {
                "Make an app."
            }
        }
        Mode::Thinking => "Creating...",
        Mode::Review => "Ready to install.",
        Mode::WorkflowReview => "Review the process.",
        Mode::BehaviorReview => "Review behavior.",
        Mode::Behaviors => "Do more.",
        Mode::BehaviorPreview => "Review changes.",
        Mode::Workflow => a.map_or("Process", |a| a.spec.title.text()),
        Mode::Form | Mode::Records | Mode::Table => a.map_or("Your app", |a| a.spec.title.text()),
        Mode::Notice => "One more thing.",
        Mode::Transfer => "Moving your data.",
        Mode::ImportReview => "Review your data.",
    };
    header(s, u, title);
    match u.mode {
        Mode::Home | Mode::Search => {
            frame(s, 24, 146, 432, 48, QUIET);
            text(
                s,
                if u.mode == Mode::Search && !u.search.text().is_empty() {
                    u.search.text()
                } else {
                    "/ search your apps"
                },
                44,
                161,
                if u.mode == Mode::Search { LIME } else { QUIET },
            );
            let mut count = 0;
            for (slot, app) in u.runtime.entries().filter(|(i, _)| u.app_matches(*i)) {
                let x = 44 + count % 2 * 216;
                let y = 242 + count / 2 * 178;
                count += 1;
                let c = if slot == u.selected { LIME } else { WHITE };
                let shape = app
                    .spec
                    .id
                    .text()
                    .bytes()
                    .fold(0usize, |n, b| n + usize::from(b))
                    % 3;
                icon(s, x + 53, y, shape, c);
                let title = app.spec.title.text();
                let split = title.len().min(23);
                text(
                    s,
                    &title[..split],
                    x as i32 + (176 - split as i32 * 7) / 2,
                    y as i32 + 88,
                    c,
                );
                if split < title.len() {
                    text(s, &title[split..], x as i32 + 24, y as i32 + 106, QUIET);
                }
                if slot == u.selected {
                    text(s, "_", x as i32 + 84, y as i32 + 125, LIME);
                }
            }
            if count < 4 {
                let x = 44 + count % 2 * 216;
                let y = 242 + count / 2 * 178;
                icon(s, x + 53, y, 3, QUIET);
                text(s, "Create", x as i32 + 67, y as i32 + 88, QUIET);
            }
            if u.mode == Mode::Search && count == 0 {
                center(s, "No matching apps", 622, Size::Small, QUIET);
            }
            action(
                s,
                if count == 0 {
                    "+ Create your first app"
                } else {
                    "Enter / Open app"
                },
            );
        }
        Mode::Compose | Mode::Thinking => {
            // An original line-art spark: decorative, never fabricated telemetry.
            for (i, line) in [
                "      .      ",
                "  .   |   .  ",
                "   \\  |  /   ",
                "----  +  ----",
                "   /  |  \\   ",
                "  '   |   '  ",
            ]
            .iter()
            .enumerate()
            {
                center(
                    s,
                    line,
                    162 + i as i32 * 18,
                    Size::Small,
                    if i == 3 { LIME } else { QUIET },
                );
            }
            frame(
                s,
                24,
                328,
                432,
                220,
                if u.mode == Mode::Thinking {
                    QUIET
                } else {
                    LIME
                },
            );
            text(s, ">", 40, 350, LIME);
            wrap(
                s,
                if u.prompt.text().is_empty() {
                    "What do you want to create?_"
                } else {
                    u.prompt.text()
                },
                389,
                6,
            );
            center(
                s,
                if u.target().is_some() {
                    "Your input records stay."
                } else {
                    "Your idea. Your app."
                },
                588,
                Size::Small,
                QUIET,
            );
            if u.mode == Mode::Thinking && !u.message.text().is_empty() {
                wrap(s, u.message.text(), 628, 2);
            }
            action(
                s,
                if u.mode == Mode::Thinking {
                    "Waiting for Meta..."
                } else {
                    "Enter / Create"
                },
            );
        }
        Mode::Review => {
            center(s, u.pending.title.text(), 175, Size::Body, WHITE);
            for (i, f) in u.pending.fields().iter().enumerate() {
                let y = 222 + i as i32 * 56;
                text(s, f.label.text(), 42, y, WHITE);
                text(s, f.kind.name(), 371, y, QUIET);
                // Formula text is bounded printable ASCII; both slices are valid.
                let expression = f.formula.text();
                let split = expression.len().min(48);
                text(s, &expression[..split], 42, y + 18, LIME);
                text(s, &expression[split..], 42, y + 34, LIME);
            }
            text(
                s,
                if u.pending.capability() == Capability::Brightness {
                    "Allow: change display brightness"
                } else {
                    "Local records only"
                },
                42,
                577,
                LIME,
            );
            wrap(s, u.message.text(), 606, 3);
            action(
                s,
                if u.pending.workflow.enabled() {
                    "Review process >"
                } else if !u.pending.behaviors.is_empty() {
                    "Review behavior >"
                } else {
                    "+ Install app"
                },
            );
        }
        Mode::Form => {
            if let Some(a) = a {
                text(
                    s,
                    if u.editing.is_some() {
                        "/ edit record"
                    } else if a.spec.capability() == Capability::Brightness {
                        "/ display control"
                    } else {
                        "/ new record"
                    },
                    26,
                    154,
                    QUIET,
                );
                let calculated = a.spec.materialize(&u.draft).ok();
                for (i, f) in a.spec.fields().iter().enumerate() {
                    let y = 190 + i * 72;
                    frame(
                        s,
                        24,
                        y,
                        432,
                        64,
                        if i == u.focus { LIME } else { 0x26392e },
                    );
                    text(s, f.label.text(), 42, y as i32 + 6, QUIET);
                    let value = if f.kind == Kind::Calculated {
                        text(s, "= auto", 362, y as i32 + 6, LIME);
                        calculated.as_ref().map_or("--", |v| v[i].text())
                    } else if f.kind == Kind::Check {
                        if u.draft[i].text() == "true" {
                            "[x]"
                        } else {
                            "[ ]"
                        }
                    } else if u.draft[i].text().is_empty() {
                        "_"
                    } else {
                        u.draft[i].text()
                    };
                    t::body(s, value, 42, y as i32 + 28, WHITE);
                }
                if !u.message.text().is_empty() {
                    wrap(s, u.message.text(), 638, 2);
                } else {
                    text(
                        s,
                        if a.spec.capability() == Capability::Brightness {
                            "Enter apply   F6 reset   F2 evolve"
                        } else {
                            "Tab fields   F3 records   F5 table"
                        },
                        42,
                        651,
                        QUIET,
                    );
                }
                if a.spec.capability() == Capability::Brightness {
                    text(s, "20..100 percent / software dimming", 42, 283, QUIET);
                    frame(s, 24, 330, 432, 48, QUIET);
                    center(s, "Reset to 100% / F6", 342, Size::Body, WHITE);
                    text(s, "Current display", 42, 408, QUIET);
                    number(s, i64::from(u.brightness.percent()), 365, 408);
                    t::rect(s, 42, 450, 396, 32, 0xffffff);
                    text(s, "QEMU pixels / no physical backlight", 42, 503, QUIET);
                    action(s, "Apply brightness");
                } else {
                    action(
                        s,
                        if u.editing
                            .and_then(|row| a.state(row))
                            .is_some_and(|state| a.spec.workflow.terminal(state))
                        {
                            "Final state / record locked"
                        } else {
                            "Save record"
                        },
                    );
                }
            }
        }
        Mode::Records => {
            if let Some(a) = a {
                text(
                    s,
                    if u.filtered {
                        a.spec.filter_label.text()
                    } else {
                        "/ all records"
                    },
                    26,
                    154,
                    LIME,
                );
                let mut visible = 0;
                for row in 0..a.count() {
                    if !u.visible(row) {
                        continue;
                    }
                    let v = a.row(row).unwrap();
                    let y = 214 + visible * 50;
                    visible += 1;
                    if row == u.row {
                        frame(s, 24, y, 432, 44, QUIET);
                    }
                    let mark = if let Some(i) = a.spec.check_field() {
                        if v[i].text() == "true" { "[x]" } else { "[ ]" }
                    } else {
                        ">"
                    };
                    text(s, mark, 39, y as i32 + 13, LIME);
                    let headline = v[0].text();
                    t::body(
                        s,
                        if a.spec.workflow.enabled() {
                            &headline[..headline.len().min(20)]
                        } else {
                            headline
                        },
                        78,
                        y as i32 + 8,
                        WHITE,
                    );
                    if a.spec.workflow.enabled() {
                        text(s, a.state(row).unwrap().text(), 345, y as i32 + 13, LIME);
                    }
                }
                if visible == 0 {
                    center(s, "Nothing here yet_", 321, Size::Body, QUIET);
                }
                text(
                    s,
                    "N new   F4 filter   F5 table   F2 evolve",
                    42,
                    651,
                    QUIET,
                );
                action(s, "Open record");
            }
        }
        Mode::Table => {
            if let Some(a) = a {
                text(s, "records", 42, 154, QUIET);
                text(s, "[ table ]", 184, 154, LIME);
                frame(s, 24, 192, 432, 446, QUIET);
                for (column, f) in a
                    .spec
                    .fields()
                    .iter()
                    .skip(u.columns * 3)
                    .take(3)
                    .enumerate()
                {
                    let x = 67 + column as i32 * 128;
                    let title = f.label.text();
                    text(s, &title[..title.len().min(16)], x, 207, LIME);
                }
                let mut visible = 0;
                for row in 0..a.count() {
                    if !u.visible(row) {
                        continue;
                    }
                    let y = 244 + visible * 40;
                    visible += 1;
                    let values = a.row(row).unwrap();
                    if row == u.row {
                        t::rect(s, 25, y - 2, 430, 35, 0x102019);
                    }
                    let number = [b'1' + row as u8];
                    text(
                        s,
                        core::str::from_utf8(&number).unwrap_or("?"),
                        39,
                        y as i32 + 5,
                        QUIET,
                    );
                    for (column, v) in values
                        .iter()
                        .skip(u.columns * 3)
                        .take(a.spec.fields().len().saturating_sub(u.columns * 3).min(3))
                        .enumerate()
                    {
                        let value = v.text();
                        let x = 67 + column as i32 * 128;
                        text(s, &value[..value.len().min(16)], x, y as i32 + 5, WHITE);
                    }
                }
                text(
                    s,
                    if u.filtered {
                        "Pending totals"
                    } else {
                        "All-row totals"
                    },
                    42,
                    575,
                    QUIET,
                );
                for column in 0..3 {
                    if let Some(sum) = a.sum(u.columns * 3 + column, u.filtered) {
                        number(s, sum, 67 + column as i32 * 128, 606);
                    }
                }
                if u.message.text().is_empty() {
                    text(s, "N new   F4 filter", 42, 651, QUIET);
                } else {
                    text(s, "Rows saved locally.", 42, 651, QUIET);
                }
                text(
                    s,
                    if a.spec.fields().len() <= 3 {
                        "1 / 1"
                    } else if u.columns == 0 {
                        "1 / 2 >"
                    } else {
                        "< 2 / 2"
                    },
                    371,
                    651,
                    LIME,
                );
                action(s, "Edit row");
            }
        }
        Mode::Workflow | Mode::WorkflowReview => {
            let review = u.mode == Mode::WorkflowReview;
            let spec = if review {
                Some(&u.pending)
            } else {
                a.map(|a| &a.spec)
            };
            if let Some(spec) = spec {
                let state = if review {
                    spec.workflow.initial
                } else {
                    a.and_then(|a| a.state(u.row))
                        .unwrap_or(spec.workflow.initial)
                };
                if !review && let Some(values) = a.and_then(|a| a.row(u.row)) {
                    center(s, values[0].text(), 192, Size::Body, WHITE);
                }
                text(
                    s,
                    if review {
                        "Initial state:"
                    } else {
                        "Current state:"
                    },
                    42,
                    213,
                    QUIET,
                );
                text(s, state.text(), 180, 213, LIME);
                for (i, step) in spec.workflow.actions().iter().enumerate() {
                    let y = 238 + i * 84;
                    let allowed = review
                        || a.and_then(|a| a.row(u.row))
                            .is_some_and(|v| step.allowed(spec, v, state).is_ok());
                    frame(
                        s,
                        24,
                        y,
                        432,
                        74,
                        if !review && i == u.focus { LIME } else { QUIET },
                    );
                    text(
                        s,
                        step.label.text(),
                        42,
                        y as i32 + 7,
                        if allowed { WHITE } else { QUIET },
                    );
                    text(s, step.from.text(), 42, y as i32 + 29, QUIET);
                    text(s, "->", 140, y as i32 + 29, QUIET);
                    text(s, step.to.text(), 165, y as i32 + 29, QUIET);
                    if step.guard_op.text() != "none" {
                        text(s, step.guard_field.text(), 42, y as i32 + 49, LIME);
                        text(
                            s,
                            match step.guard_op.text() {
                                "le" => "<=",
                                "ge" => ">=",
                                _ => "=",
                            },
                            140,
                            y as i32 + 49,
                            LIME,
                        );
                        number(s, i64::from(step.guard_value), 180, y as i32 + 49);
                        if !review
                            && let Some(values) = a.and_then(|a| a.row(u.row))
                            && let Some(field) =
                                spec.fields().iter().position(|f| f.key == step.guard_field)
                            && let Ok(value) = values[field].text().parse::<i32>()
                        {
                            text(s, "now", 270, y as i32 + 49, QUIET);
                            number(s, i64::from(value), 304, y as i32 + 49);
                        }
                    }
                }
                if review {
                    wrap(
                        s,
                        "Actions change this record's state. Final states lock record editing.",
                        590,
                        3,
                    );
                    action(
                        s,
                        if spec.behaviors.is_empty() {
                            "+ Install app"
                        } else {
                            "Review behavior >"
                        },
                    );
                } else {
                    let message = match u.message.text() {
                        "workflow_guard_failed" => "Condition not met. Update the record first.",
                        "workflow_wrong_state" => {
                            "This action is not available in the current state."
                        }
                        "" => "Tap an action. F3 returns to records.",
                        other => other,
                    };
                    wrap(s, message, 590, 3);
                    action(s, "Run selected action");
                }
            }
        }
        Mode::BehaviorReview => {
            if let Some(button) = u.pending.behaviors.buttons().get(u.focus) {
                text(s, "Button", 42, 147, QUIET);
                number(s, (u.focus + 1) as i64, 102, 147);
                text(s, "of", 125, 147, QUIET);
                number(s, u.pending.behaviors.buttons().len() as i64, 150, 147);
                center(s, button.label.text(), 174, Size::Body, WHITE);
                for (i, step) in button.steps().iter().enumerate() {
                    let y = 210 + i * 64;
                    frame(s, 24, y, 432, 60, QUIET);
                    number(s, (i + 1) as i64, 38, y as i32 + 5);
                    text(s, step.op.text(), 65, y as i32 + 5, WHITE);
                    text(s, step.field.text(), 163, y as i32 + 5, LIME);
                    text(s, step.compare.text(), 275, y as i32 + 5, WHITE);
                    if step.skip != 0 {
                        text(s, "skip", 335, y as i32 + 5, QUIET);
                        number(s, step.skip as i64, 383, y as i32 + 5);
                    }
                    let value = step.value.text();
                    let split = value.len().min(48);
                    text(s, &value[..split], 42, y as i32 + 23, LIME);
                    text(s, &value[split..], 42, y as i32 + 40, LIME);
                }
                wrap(
                    s,
                    "One saved record. In order. False if skips steps. Any failure saves nothing.",
                    613,
                    3,
                );
                action(
                    s,
                    if u.focus + 1 < u.pending.behaviors.buttons().len() {
                        "Review next button >"
                    } else {
                        "+ Install app"
                    },
                );
            }
        }
        Mode::Behaviors => {
            if let Some(a) = a {
                if let Some(values) = a.row(u.row) {
                    center(s, values[0].text(), 194, Size::Body, WHITE);
                }
                for (i, button) in a.spec.behaviors.buttons().iter().enumerate() {
                    let y = 238 + i * 94;
                    frame(s, 24, y, 432, 84, if i == u.focus { LIME } else { QUIET });
                    text(s, button.label.text(), 42, y as i32 + 13, WHITE);
                    number(s, button.steps().len() as i64, 42, y as i32 + 44);
                    text(
                        s,
                        "steps / preview before applying",
                        65,
                        y as i32 + 44,
                        QUIET,
                    );
                }
                wrap(
                    s,
                    if u.message.text().is_empty() {
                        "Choose a button. Changes are checked together before they are saved."
                    } else {
                        u.message.text()
                    },
                    490,
                    4,
                );
                if let Some(error) = u.behavior_error {
                    text(s, "Failed step:", 42, 590, QUIET);
                    number(s, i64::from(error.step), 160, 590);
                    text(s, "Nothing saved.", 42, 614, WHITE);
                }
                action(s, "Preview changes >");
            }
        }
        Mode::BehaviorPreview => {
            if let Some(a) = a
                && let Some(before) = a.row(u.row)
            {
                text(s, u.behavior_label(), 42, 194, LIME);
                for (i, field) in a.spec.fields().iter().enumerate() {
                    let y = 224 + i as i32 * 64;
                    text(s, field.label.text(), 42, y, WHITE);
                    text(s, "was", 42, y + 19, QUIET);
                    text(s, before[i].text(), 84, y + 19, QUIET);
                    text(s, "now", 42, y + 36, LIME);
                    text(
                        s,
                        u.draft[i].text(),
                        84,
                        y + 36,
                        if before[i] == u.draft[i] { QUIET } else { LIME },
                    );
                }
                wrap(
                    s,
                    if u.message.text().is_empty() {
                        "No changes saved yet. Confirm together, or leave this screen to cancel."
                    } else {
                        u.message.text()
                    },
                    617,
                    3,
                );
                action(s, "Apply all changes");
            }
        }
        Mode::Transfer => {
            wrap(
                s,
                "Waiting for the selected exchange folder. Esc cancels.",
                260,
                3,
            );
        }
        Mode::ImportReview => {
            if let Some(a) = a {
                center(s, a.spec.title.text(), 160, Size::Body, WHITE);
                text(s, "Incoming row", 42, 203, QUIET);
                number(s, (u.row + 1) as i64, 176, 203);
                text(s, "of", 199, 203, QUIET);
                number(s, u.import.len() as i64, 224, 203);
                if let Ok(values) = u.import.values(&a.spec, u.row) {
                    for (i, field) in a.spec.fields().iter().enumerate() {
                        let y = 250 + i as i32 * 50;
                        text(s, field.label.text(), 42, y, QUIET);
                        t::body(s, values[i].text(), 42, y + 18, WHITE);
                    }
                }
                wrap(
                    s,
                    "Append all rows. Existing records stay. Tab / arrows: inspect each row.",
                    580,
                    3,
                );
                action(s, "+ Import rows");
            }
        }
        Mode::Notice => {
            wrap(s, u.message.text(), 260, 8);
            action(s, "Back home");
        }
    }
    if matches!(
        u.mode,
        Mode::Form
            | Mode::Records
            | Mode::Table
            | Mode::Workflow
            | Mode::Behaviors
            | Mode::BehaviorPreview
    ) && a.is_some_and(|a| a.spec.capability() == Capability::Records)
    {
        t::rect(s, 24, 146, 432, 36, 0x060908);
        if a.is_some_and(|a| !a.spec.behaviors.is_empty()) {
            text(s, "records", 30, 154, QUIET);
            text(s, "table", 110, 154, QUIET);
            if a.is_some_and(|a| a.spec.workflow.enabled()) {
                text(s, "flow", 180, 154, LIME);
            }
            text(s, "do", 252, 154, LIME);
            text(s, "import", 309, 154, LIME);
            text(s, "export", 390, 154, LIME);
        } else {
            text(
                s,
                if u.mode == Mode::Records {
                    "[records]"
                } else {
                    "records"
                },
                30,
                154,
                QUIET,
            );
            text(
                s,
                if u.mode == Mode::Table {
                    "[table]"
                } else {
                    "table"
                },
                125,
                154,
                QUIET,
            );
            if a.is_some_and(|a| a.spec.workflow.enabled()) {
                text(s, "actions", 210, 154, LIME);
            }
            text(s, "import", 300, 154, LIME);
            text(s, "export", 390, 154, LIME);
        }
    }
    if !u.storage.text().is_empty() {
        center(s, u.storage.text(), 672, Size::Small, QUIET);
    }
    nav(s);
    s.present();
}

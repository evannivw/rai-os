//! Declarative app presenter: observes state without mutating records or policy.
use super::{Display, terminal as t};
use crate::{
    app_ui::{Controller, Mode},
    microapp::Kind,
};
pub fn render(s: &mut Display, u: &Controller) {
    let app = u.runtime.app(u.selected);
    let title = match u.mode {
        Mode::Home => "Build what you need.",
        Mode::Compose => "Describe a tool.",
        Mode::Thinking => "Compiling intent_",
        Mode::Review => "Inspect. Then install.",
        Mode::Form | Mode::Records => app.map_or("Microapps", |a| a.spec.title.text()),
        Mode::Notice => "A boundary, clearly.",
    };
    t::base(
        s,
        "/ APPS     DECLARATIVE RUNTIME",
        title,
        "Your tools / local execution / records in RAM",
        u.live,
    );
    match u.mode {
        Mode::Home => {
            t::text(s, "INSTALLED / YOUR TOOLS", 26, 194, t::MUTED);
            for (slot, a) in u.runtime.entries() {
                let y = 224 + slot * 80;
                if slot == u.selected {
                    t::rect(s, 24, y, 432, 68, t::PANEL);
                    t::rect(s, 24, y, 3, 68, t::ACCENT);
                }
                t::body(s, a.spec.title.text(), 42, y as i32 + 10, t::INK);
                t::text(s, a.spec.id.text(), 42, y as i32 + 40, t::MUTED);
            }
            if u.runtime.entries().count() == 0 {
                t::body(s, "[ ] No apps installed yet", 30, 252, t::INK);
                t::wrap(
                    s,
                    "A prompt becomes a form, a record view and a small tool you can use without another AI call.",
                    300,
                    3,
                    t::MUTED,
                );
            }
            t::prompt(
                s,
                "> CREATE / F2",
                "Describe a tool. Review its fields. Make it yours.",
            );
            t::button(
                s,
                "Enter / Open selected",
                "F2 new app   Tab select   Esc home   RAM only",
            );
        }
        Mode::Compose | Mode::Thinking => {
            t::text(
                s,
                if u.target().is_some() {
                    "EVOLVE / EXISTING RECORDS PRESERVED"
                } else {
                    "CREATE / ONE PURPOSE, YOUR STRUCTURE"
                },
                26,
                194,
                t::ACCENT,
            );
            for (i, (n, v)) in [
                ("01 / DEFINE", "Text, numbers, checks. Up to six fields."),
                ("02 / REVIEW", "Only local records. No ambient access."),
                ("03 / USE", "Save, edit and filter inside RAI-OS."),
            ]
            .iter()
            .enumerate()
            {
                let y = 240 + i as i32 * 82;
                t::text(s, n, 30, y, t::MUTED);
                t::body(s, v, 30, y + 24, t::INK);
            }
            t::prompt(
                s,
                "> YOUR INTENT",
                if u.prompt.text().is_empty() {
                    "Create an inspection app with equipment, notes, a done checkbox and a pending view."
                } else {
                    u.prompt.text()
                },
            );
            t::button(
                s,
                if u.mode == Mode::Thinking {
                    "Waiting for Meta proposal..."
                } else {
                    "Enter / Generate app"
                },
                "Esc cancel    ASCII input / 128 characters",
            );
        }
        Mode::Review => {
            t::body(s, u.pending.title.text(), 30, 193, t::ACCENT);
            for (i, f) in u.pending.fields().iter().enumerate() {
                let y = 235 + i as i32 * 40;
                t::text(s, f.kind.name(), 30, y, t::MUTED);
                t::body(s, f.label.text(), 112, y - 4, t::INK);
                t::divider(s, y as usize + 28);
            }
            t::text(s, "CAPABILITY / records.local", 30, 490, t::ACCENT);
            t::text(
                s,
                "Schema checked. Existing values retained.",
                30,
                516,
                t::MUTED,
            );
            t::prompt(s, "> PLAN / AWAITING YOUR CONFIRMATION", u.message.text());
            t::button(
                s,
                "Enter / Install this app",
                "Esc discard    No installation before approval",
            );
        }
        Mode::Form => {
            if let Some(a) = app {
                t::text(
                    s,
                    if u.editing.is_some() {
                        "EDIT RECORD / LOCAL EXECUTION"
                    } else {
                        "NEW RECORD / LOCAL EXECUTION"
                    },
                    26,
                    184,
                    t::ACCENT,
                );
                for (i, f) in a.spec.fields().iter().enumerate() {
                    let y = 214 + i * 52;
                    if i == u.focus {
                        t::rect(s, 24, y, 432, 48, t::PANEL);
                        t::rect(s, 24, y, 3, 48, t::ACCENT);
                    }
                    t::text(s, f.label.text(), 36, y as i32 + 2, t::MUTED);
                    let value = if f.kind == Kind::Check {
                        if u.draft[i].text() == "true" {
                            "[x] true"
                        } else {
                            "[ ] false"
                        }
                    } else if u.draft[i].text().is_empty() {
                        "_"
                    } else {
                        u.draft[i].text()
                    };
                    t::body(s, value, 36, y as i32 + 21, t::INK);
                }
                t::prompt(
                    s,
                    "> APP / RECORDS STAY IN THIS NAMESPACE",
                    if u.message.text().is_empty() {
                        "Tab changes fields. Space toggles a checkbox. Enter validates and saves the complete record."
                    } else {
                        u.message.text()
                    },
                );
                t::button(
                    s,
                    "Enter / Save record",
                    "Tab field   F2 evolve   F3 records   Esc home",
                );
            }
        }
        Mode::Records => {
            if let Some(a) = app {
                t::text(
                    s,
                    if u.filtered {
                        a.spec.filter_label.text()
                    } else {
                        "ALL RECORDS"
                    },
                    26,
                    184,
                    t::ACCENT,
                );
                let mut visible = 0;
                for row in 0..a.count() {
                    if !u.visible(row) {
                        continue;
                    }
                    let values = a.row(row).unwrap();
                    let y = 214 + visible * 39;
                    visible += 1;
                    if row == u.row {
                        t::rect(s, 24, y, 432, 35, t::PANEL);
                        t::rect(s, 24, y, 3, 35, t::ACCENT);
                    }
                    let checked = a
                        .spec
                        .check_field()
                        .is_some_and(|i| values[i].text() == "true");
                    t::text(
                        s,
                        if checked { "[x]" } else { "[ ]" },
                        36,
                        y as i32 + 9,
                        t::ACCENT,
                    );
                    t::body(s, values[0].text(), 70, y as i32 + 5, t::INK);
                }
                if visible == 0 {
                    t::body(s, "No records in this view_", 30, 244, t::MUTED);
                }
                t::prompt(
                    s,
                    "> RECORD / SELECTED DETAIL",
                    if let Some(row) = a.row(u.row).filter(|_| u.visible(u.row)) {
                        row.get(1).map_or("", |v| v.text())
                    } else {
                        "Press N to add a record."
                    },
                );
                t::button(
                    s,
                    "Enter / Edit selected",
                    "N new   Space status   F4 filter   F2 evolve",
                );
            }
        }
        Mode::Notice => {
            t::wrap(s, u.message.text(), 220, 8, t::INK);
            t::button(s, "Enter / Back home", "No app changed. Esc home");
        }
    }
    s.present();
}

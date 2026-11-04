//! Deterministic host-only controller stress check. No QEMU, API calls or credentials.
//!
//! Run from the repository root with the pinned Rust toolchain:
//!   mkdir -p target/verification
//!   rustc --edition 2024 scripts/check_controller_stress.rs -o target/verification/controller-stress
//!   target/verification/controller-stress
//!
//! A fixed seed drives one million navigation, pointer, edit and proposal actions
//! across fresh bounded app states. Success means these actions did not panic;
//! it does not prove that every transition or generated application is correct.
#![allow(dead_code, unused_imports)]
#[path = "../src/app_ui.rs"]
mod app_ui;
#[path = "../src/microapp.rs"]
mod microapp;
use app_ui::*;
use microapp::*;
fn spec(id: &str) -> Spec {
    let mut s = Spec::new(id, "Stress records", "records.local").unwrap();
    for (key, kind, default) in [
        ("name", "text", "A"),
        ("balance", "number", "8"),
        ("amount", "number", "1"),
        ("to", "text", "B"),
        ("done", "check", "false"),
    ] {
        s.add(Field::new(key, key, kind, true, default).unwrap())
            .unwrap();
    }
    s.add(
        Field::new("twice", "twice", "calculated", true, "")
            .unwrap()
            .with_formula("$balance*2")
            .unwrap(),
    )
    .unwrap();
    s.filter("done", "Pending").unwrap();
    s.workflow = Workflow::new("new").unwrap();
    s.workflow
        .add(Action::new("close", "Close", "new", "closed", "balance", "ge", 0).unwrap())
        .unwrap();
    let mut b = Button::new("increment", "Increment", "press").unwrap();
    b.add(Step::new("calculate", "balance", "$balance+1", "", 0).unwrap())
        .unwrap();
    s.behaviors.add(b).unwrap();
    s.pages.add("Report").unwrap();
    s.pages
        .add_block(
            0,
            Block::new("metric", "Total", "balance", "", "sum", "all").unwrap(),
        )
        .unwrap();
    s.transfer = Some(Transfer::new("move", "Move", "balance", "amount", "to", "name").unwrap());
    s
}
fn seed() -> Controller {
    let mut c = Controller::new();
    let s = spec("work");
    c.runtime.install(&s, c.runtime.revision(), None).unwrap();
    for i in 0..8 {
        let mut v = c.runtime.app(0).unwrap().defaults();
        v[0] = Text::new(&format!("Row{i}")).unwrap();
        c.runtime.save(0, None, &v).unwrap();
    }
    let mut b = Spec::new("screen", "Screen", "display.brightness").unwrap();
    b.add(Field::new("level", "Level", "number", true, "80").unwrap())
        .unwrap();
    c.runtime.install(&b, c.runtime.revision(), None).unwrap();
    c.display_available = true;
    c.execute("app.open", None).unwrap();
    c
}
fn hex(s: &str) -> String {
    s.bytes().map(|b| format!("{b:02x}")).collect()
}
fn reply(c: &mut Controller, r: Reply) {
    if let Reply::Submit {
        request,
        revision,
        target,
        ..
    } = r
    {
        let id = if target.text().is_empty() {
            "third"
        } else {
            target.text()
        };
        let title = hex("Stress records");
        let arg = format!("{request}:{revision}:{id}:{title}:records.local");
        if c.execute("app.begin", Some(&arg)).is_err() {
            return;
        }
        for f in spec("work").fields() {
            let arg = format!(
                "{request}:{}:{}:{}:{}:{}",
                f.key.text(),
                f.kind.name(),
                u8::from(f.required),
                hex(f.label.text()),
                hex(f.default.text())
            );
            let _ = c.execute("app.field", Some(&arg));
            if f.kind == Kind::Calculated {
                let arg = format!("{request}:{}:{}", f.key.text(), hex(f.formula.text()));
                let _ = c.execute("app.formula", Some(&arg));
            }
        }
        let _ = c.execute("app.flow", Some(&format!("{request}:new")));
        let _ = c.execute(
            "app.action",
            Some(&format!(
                "{request}:close:{}:new:closed:balance:ge:0",
                hex("Close")
            )),
        );
        let _ = c.execute("app.ready", Some(&format!("{request}:Stress proposal")));
    }
}
fn main() {
    let mut state = 0x32abcdef_u64;
    let mut count = 0;
    for _ in 0..100 {
        let mut c = seed();
        for _ in 0..10000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let r = match state % 8 {
                0 => c.execute(
                    "app.open",
                    Some(if state & 8 == 0 { "work" } else { "screen" }),
                ),
                1 => {
                    let arg = format!("{}:{}", (state >> 16) % 500, (state >> 32) % 820);
                    c.execute("app.tap", Some(&arg))
                }
                2 => {
                    let arg = format!(
                        "{}:{}",
                        (state >> 16) % 8,
                        hex(if state & 8 == 0 { "1" } else { "B" })
                    );
                    c.execute("app.value", Some(&arg))
                }
                _ => {
                    let keys = [
                        0, 8, 9, 13, 27, 32, 47, 78, 97, 110, 127, 1001, 1002, 1003, 1004, 1005,
                        1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1016,
                    ];
                    let arg = keys[((state >> 8) as usize) % keys.len()].to_string();
                    c.execute("app.key", Some(&arg))
                }
            };
            if let Ok(r) = r {
                reply(&mut c, r)
            }
            count += 1;
        }
    }
    println!("PASS: {count} deterministic controller actions without panic");
}

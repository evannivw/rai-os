//! Host tests for update-review navigation and admission; no QEMU or model call.
//! Run: rustc --edition 2024 --test scripts/check_update_review.rs -o target/update-review-tests
//! Then: target/update-review-tests
#![allow(dead_code, unused_imports)]
#[path = "../src/app_ui.rs"]
mod app_ui;
#[path = "../src/microapp.rs"]
mod microapp;
use app_ui::{Controller, Mode, Reply};
use microapp::{Field, Spec, Text};

fn spec(formula: &str) -> Spec {
    let mut spec = Spec::new("calculator", "My calculator", "records.local").unwrap();
    spec.add(Field::new("qty", "Quantity", "number", true, "2").unwrap())
        .unwrap();
    spec.add(
        Field::new("total", "Total", "calculated", true, "")
            .unwrap()
            .with_formula(formula)
            .unwrap(),
    )
    .unwrap();
    spec
}
fn seed() -> Controller {
    let mut controller = Controller::new();
    controller
        .runtime
        .install(&spec("$qty*2"), 1, None)
        .unwrap();
    let values = controller.runtime.app(0).unwrap().defaults();
    controller.runtime.save(0, None, &values).unwrap();
    controller
}
fn key(controller: &mut Controller, key: u32) {
    controller
        .execute("app.key", Some(&key.to_string()))
        .unwrap();
}
fn hex(value: &str) -> String {
    value.bytes().map(|byte| format!("{byte:02x}")).collect()
}
fn propose(controller: &mut Controller, spec: &Spec, target: Option<&str>) -> u32 {
    controller.execute("app.open", target).unwrap();
    key(controller, 1002);
    key(controller, u32::from(b'E'));
    let Reply::Submit {
        request, revision, ..
    } = controller.execute("app.key", Some("13")).unwrap()
    else {
        panic!("Expected a proposal request")
    };
    controller
        .execute(
            "app.begin",
            Some(&format!(
                "{request}:{revision}:{}:{}:records.local",
                spec.id.text(),
                hex(spec.title.text())
            )),
        )
        .unwrap();
    for field in spec.fields() {
        controller
            .execute(
                "app.field",
                Some(&format!(
                    "{request}:{}:{}:{}:{}:{}",
                    field.key.text(),
                    field.kind.name(),
                    u8::from(field.required),
                    hex(field.label.text()),
                    hex(field.default.text())
                )),
            )
            .unwrap();
        if !field.formula.text().is_empty() {
            controller
                .execute(
                    "app.formula",
                    Some(&format!(
                        "{request}:{}:{}",
                        field.key.text(),
                        hex(field.formula.text())
                    )),
                )
                .unwrap();
        }
    }
    controller
        .execute("app.ready", Some(&format!("{request}:Test proposal")))
        .unwrap();
    assert!(controller.mode == Mode::Review);
    request
}

#[test]
fn update_review_pages_without_mutation_then_uses_existing_approval() {
    let mut controller = seed();
    let revision = controller.runtime.revision();
    propose(&mut controller, &spec("$qty*3"), Some("calculator"));
    assert_eq!(controller.focus, 1, "First changed field is selected");
    for (input, expected) in [(9, 0), (1009, 1), (1008, 0), (1005, 1), (1006, 0)] {
        key(&mut controller, input);
        assert_eq!(controller.focus, expected);
    }
    controller.execute("app.tap", Some("320:570")).unwrap();
    assert_eq!(controller.focus, 1);
    controller.execute("app.tap", Some("100:570")).unwrap();
    assert_eq!(controller.focus, 0);
    assert_eq!(controller.runtime.revision(), revision);
    assert_eq!(
        controller.runtime.app(0).unwrap().row(0).unwrap()[1].text(),
        "4"
    );
    key(&mut controller, 13);
    let app = controller.runtime.app(0).unwrap();
    assert_eq!(app.version, 2);
    assert_eq!(app.row(0).unwrap()[0].text(), "2");
    assert_eq!(app.row(0).unwrap()[1].text(), "6");
    assert_eq!(controller.message.text(), "App updated.");
}

#[test]
fn cancellation_and_later_revision_never_install_the_reviewed_proposal() {
    let mut controller = seed();
    let request = propose(&mut controller, &spec("$qty*3"), Some("calculator"));
    key(&mut controller, 27);
    assert_eq!(controller.runtime.app(0).unwrap().version, 1);
    assert!(
        controller
            .execute("app.ready", Some(&format!("{request}:Late response")))
            .err()
            == Some("app_stale_request")
    );
    propose(&mut controller, &spec("$qty*3"), Some("calculator"));
    let mut values = *controller.runtime.app(0).unwrap().row(0).unwrap();
    values[0] = Text::new("9").unwrap();
    controller.runtime.save(0, Some(0), &values).unwrap();
    assert!(controller.execute("app.key", Some("13")).err() == Some("app_stale_revision"));
    let app = controller.runtime.app(0).unwrap();
    assert_eq!(app.version, 1);
    assert_eq!(app.row(0).unwrap()[0].text(), "9");
    assert_eq!(app.row(0).unwrap()[1].text(), "18");
}

#[test]
fn added_field_and_metadata_change_are_selected_without_claiming_formula_change() {
    let mut controller = seed();
    let mut added = spec("$qty*2");
    added
        .add(Field::new("note", "Note", "text", false, "new").unwrap())
        .unwrap();
    propose(&mut controller, &added, Some("calculator"));
    assert_eq!(controller.focus, 2);
    key(&mut controller, 13);
    assert_eq!(
        controller.runtime.app(0).unwrap().row(0).unwrap()[2].text(),
        "new"
    );
    let mut renamed = Spec::new("calculator", "My calculator", "records.local").unwrap();
    for field in added.fields() {
        let mut field = *field;
        if field.key.text() == "qty" {
            field.label = Text::new("Items").unwrap();
        }
        renamed.add(field).unwrap();
    }
    propose(&mut controller, &renamed, Some("calculator"));
    assert_eq!(controller.focus, 0);
    assert_eq!(
        controller.runtime.app(0).unwrap().spec.fields()[0]
            .label
            .text(),
        "Quantity"
    );
    key(&mut controller, 27);
    propose(&mut controller, &added, Some("calculator"));
    assert_eq!(
        controller.focus, 0,
        "Unchanged fields have a stable first page"
    );
}

#[test]
fn first_install_keeps_original_review_behavior() {
    let mut controller = Controller::new();
    propose(&mut controller, &spec("$qty*2"), None);
    key(&mut controller, 1008);
    assert_eq!(controller.focus, 0);
    key(&mut controller, 13);
    assert_eq!(controller.message.text(), "App installed.");
}

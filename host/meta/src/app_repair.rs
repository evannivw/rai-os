//! Narrow correction policy for decoded app proposals. Never installs an app.
//! Repair may change arithmetic expressions, not the surrounding app authority.
use crate::apps::{self, Plan, Request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

pub const MAX_ATTEMPTS: usize = 3;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Feedback {
    pub attempt: usize,
    pub code: String,
    pub proposal: Plan,
}

pub fn repairable(code: &str) -> bool {
    matches!(
        code,
        "formula_syntax" | "formula_unknown_field" | "formula_limit"
    )
}

pub fn hint(code: &str) -> &'static str {
    match code {
        "formula_syntax" => {
            "Use an arithmetic expression only: integer literals, $field references, + - * /, unary minus and parentheses. Every field reference starts with $. Do not put an assignment or destination name before the expression. The step's field property already selects the destination."
        }
        "formula_unknown_field" => {
            "Use only existing numeric field keys, each prefixed by $. A calculated field may reference only earlier numeric/calculated fields. A calculate step may reference any current numeric/calculated field. Do not add, rename or reorder fields to make a reference valid."
        }
        "formula_limit" => {
            "Shorten the arithmetic expression within 64 ASCII bytes, 32 arithmetic instructions and 8 parser nesting levels. Do not change field definitions or remove program steps to meet the bound."
        }
        _ => "",
    }
}

pub fn validate_feedback(feedback: &Feedback, request: &Request) -> Result<(), &'static str> {
    if !(2..=MAX_ATTEMPTS).contains(&feedback.attempt) {
        return Err("app_repair_attempt_limit");
    }
    if feedback.proposal.request_id != request.request_id
        || feedback.proposal.base_revision != request.base_revision
    {
        return Err("stale_app_repair");
    }
    if !repairable(&feedback.code) {
        return Err("app_repair_not_allowed");
    }
    // The diagnostic is data, not trusted authority. Reproduce its exact error
    // using the original immutable request before calling any external model.
    if apps::validate(&feedback.proposal, request) != Err(feedback.code.as_str()) {
        return Err("app_repair_feedback_mismatch");
    }
    Ok(())
}

pub fn read(path: &Path, request: &Request) -> Result<Feedback, String> {
    let feedback: Feedback = serde_json::from_str(&crate::config::bounded_file(path, 16384)?)
        .map_err(|_| "invalid_app_feedback")?;
    validate_feedback(&feedback, request)?;
    Ok(feedback)
}

fn fixed_structure(proposal: &Plan) -> Value {
    let mut value = serde_json::to_value(proposal).expect("serializable app proposal");
    value.as_object_mut().unwrap().remove("reason");
    if let Some(spec) = value.get_mut("spec").and_then(Value::as_object_mut) {
        if let Some(fields) = spec.get_mut("fields").and_then(Value::as_array_mut) {
            for field in fields {
                if field["kind"] == "calculated" {
                    field["formula"] = Value::String(String::new());
                }
            }
        }
        if let Some(buttons) = spec.get_mut("behaviors").and_then(Value::as_array_mut) {
            for button in buttons {
                if let Some(steps) = button.get_mut("steps").and_then(Value::as_array_mut) {
                    for step in steps {
                        if step["op"] == "calculate" {
                            step["value"] = Value::String(String::new());
                        }
                    }
                }
            }
        }
    }
    value
}

pub fn check_scope(previous: &Plan, candidate: &Plan) -> Result<(), &'static str> {
    if previous.decision != "install"
        || previous.spec.is_none()
        || candidate.request_id != previous.request_id
        || candidate.base_revision != previous.base_revision
    {
        return Err("app_repair_scope_denied");
    }
    // A model can stop rather than manufacture a partial application. The caller
    // still validates the decision, empty spec and reason before publishing it.
    if matches!(candidate.decision.as_str(), "clarify" | "decline") && candidate.spec.is_none() {
        return Ok(());
    }
    if fixed_structure(previous) != fixed_structure(candidate) {
        return Err("app_repair_scope_denied");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request() -> Request {
        apps::request(r#"{"request_id":4,"base_revision":7,"goal":"Move a count with a precondition","target":null,"app_ids":[],"record_count":0}"#).unwrap()
    }
    fn invalid() -> Plan {
        serde_json::from_value(json!({
            "request_id":4,"base_revision":7,"decision":"install","reason":"A composed count operation.",
            "spec":{"id":"counter","title":"Counter","capability":"records.local",
                "fields":[
                    {"key":"stock","label":"Stock","kind":"number","required":true,"default":"0"},
                    {"key":"moved","label":"Moved","kind":"number","required":true,"default":"0"},
                    {"key":"total","label":"Total","kind":"calculated","required":true,"default":"","formula":"$stock+$moved"}
                ],"filter_key":"","filter_label":"",
                "behaviors":[{"id":"move","label":"Move","event":"press","steps":[
                    {"op":"require","field":"stock","value":"1","compare":"ge","skip":0},
                    {"op":"calculate","field":"stock","value":"stock - 1","compare":"","skip":0},
                    {"op":"calculate","field":"moved","value":"$moved+1","compare":"","skip":0}
                ]}]
            }
        })).unwrap()
    }
    fn corrected(previous: &Plan) -> Plan {
        let mut candidate: Plan =
            serde_json::from_value(serde_json::to_value(previous).unwrap()).unwrap();
        candidate.spec.as_mut().unwrap().behaviors[0].steps[1].value = "$stock-1".into();
        candidate.reason = "Corrected the arithmetic syntax.".into();
        candidate
    }
    fn feedback() -> Feedback {
        Feedback {
            attempt: 2,
            code: "formula_syntax".into(),
            proposal: invalid(),
        }
    }

    #[test]
    fn feedback_must_reproduce_a_repairable_error_for_the_original_request() {
        let r = request();
        let mut f = feedback();
        assert_eq!(validate_feedback(&f, &r), Ok(()));
        f.attempt = 3;
        assert_eq!(validate_feedback(&f, &r), Ok(()));
        for attempt in [0, 1, 4, usize::MAX] {
            f.attempt = attempt;
            assert_eq!(validate_feedback(&f, &r), Err("app_repair_attempt_limit"));
        }
        f.attempt = 2;
        f.code = "formula_unknown_field".into();
        assert_eq!(
            validate_feedback(&f, &r),
            Err("app_repair_feedback_mismatch")
        );
        for code in [
            "stale_app_plan",
            "app_capability_denied",
            "meta_timeout",
            "invalid_app_plan",
            "formula_div_zero",
            "formula_overflow",
            "app_invalid_migration_default",
        ] {
            f.code = code.into();
            assert_eq!(validate_feedback(&f, &r), Err("app_repair_not_allowed"));
        }
        f.code = "formula_syntax".into();
        f.proposal.base_revision += 1;
        assert_eq!(validate_feedback(&f, &r), Err("stale_app_repair"));
        f.proposal.base_revision -= 1;
        f.proposal.request_id += 1;
        assert_eq!(validate_feedback(&f, &r), Err("stale_app_repair"));
        f.proposal = corrected(&invalid());
        assert_eq!(
            validate_feedback(&f, &r),
            Err("app_repair_feedback_mismatch")
        );
    }

    #[test]
    fn syntax_correction_is_narrow_and_requires_full_validation() {
        let original = invalid();
        let candidate = corrected(&original);
        assert_eq!(apps::validate(&original, &request()), Err("formula_syntax"));
        assert_eq!(check_scope(&original, &candidate), Ok(()));
        assert_eq!(apps::validate(&candidate, &request()), Ok(()));
        let mut still_invalid = corrected(&original);
        still_invalid.spec.as_mut().unwrap().behaviors[0].steps[1].value = "$missing-1".into();
        assert_eq!(check_scope(&original, &still_invalid), Ok(()));
        assert_eq!(
            apps::validate(&still_invalid, &request()),
            Err("formula_unknown_field")
        );
        let f = Feedback {
            attempt: 3,
            code: "formula_unknown_field".into(),
            proposal: still_invalid,
        };
        assert_eq!(validate_feedback(&f, &request()), Ok(()));
        let mut calculated = corrected(&original);
        calculated.spec.as_mut().unwrap().fields[2].formula = "($stock+$moved)".into();
        assert_eq!(check_scope(&original, &calculated), Ok(()));
        assert_eq!(apps::validate(&calculated, &request()), Ok(()));
    }

    #[test]
    fn correction_cannot_expand_button_scope() {
        let old = invalid();
        let mut candidate = corrected(&old);
        candidate.spec.as_mut().unwrap().behaviors[0].scope = "all".into();
        assert_eq!(
            check_scope(&old, &candidate),
            Err("app_repair_scope_denied")
        );
    }
    #[test]
    fn repair_cannot_weaken_or_replace_app_structure_even_while_invalid() {
        let original = invalid();
        let encoded = serde_json::to_value(&original).unwrap();
        let changes = [
            ("/request_id", json!(5)),
            ("/base_revision", json!(8)),
            ("/spec/id", json!("other")),
            ("/spec/capability", json!("display.brightness")),
            ("/spec/title", json!("Different")),
            ("/spec/fields/0/key", json!("other")),
            ("/spec/fields/0/required", json!(false)),
            ("/spec/fields/0/default", json!("10")),
            ("/spec/fields/0/kind", json!("text")),
            ("/spec/behaviors/0/steps/0/value", json!("-100")),
            ("/spec/behaviors/0/steps/0/compare", json!("le")),
            ("/spec/behaviors/0/steps/1/op", json!("set")),
            ("/spec/behaviors/0/steps/1/field", json!("moved")),
            ("/spec/behaviors/0/event", json!("timer")),
        ];
        for (path, replacement) in changes {
            let mut value = encoded.clone();
            *value.pointer_mut(path).unwrap() = replacement;
            let changed: Plan = serde_json::from_value(value).unwrap();
            assert_eq!(
                check_scope(&original, &changed),
                Err("app_repair_scope_denied"),
                "{path}"
            );
        }
        let mut value = encoded.clone();
        value["spec"]["behaviors"][0]["steps"]
            .as_array_mut()
            .unwrap()
            .remove(0);
        let removed: Plan = serde_json::from_value(value).unwrap();
        assert_eq!(
            check_scope(&original, &removed),
            Err("app_repair_scope_denied")
        );
        let mut value = encoded;
        value["spec"]["behaviors"] = json!([]);
        let removed: Plan = serde_json::from_value(value).unwrap();
        assert_eq!(
            check_scope(&original, &removed),
            Err("app_repair_scope_denied")
        );
    }

    #[test]
    fn decline_or_clarification_is_allowed_but_never_forced_to_install() {
        for decision in ["decline", "clarify"] {
            let candidate = Plan {
                request_id: 4,
                base_revision: 7,
                decision: decision.into(),
                spec: None,
                reason: "Cannot correct this without changing the requested behavior.".into(),
            };
            assert_eq!(check_scope(&invalid(), &candidate), Ok(()));
            assert_eq!(apps::validate(&candidate, &request()), Ok(()));
        }
    }

    #[test]
    fn corrections_keep_existing_update_migration_contract() {
        let original = invalid();
        let accepted = corrected(&original);
        let mut r = request();
        r.target = accepted.spec;
        r.app_ids = vec!["counter".into()];
        r.record_count = 2;
        let f = feedback();
        validate_feedback(&f, &r).unwrap();
        let repaired = corrected(&f.proposal);
        check_scope(&f.proposal, &repaired).unwrap();
        apps::validate(&repaired, &r).unwrap();
        let mut denied = corrected(&f.proposal);
        denied.spec.as_mut().unwrap().fields.remove(1);
        assert_eq!(
            check_scope(&f.proposal, &denied),
            Err("app_repair_scope_denied")
        );
        assert!(apps::validate(&denied, &r).is_err());
    }
}

//! Untrusted JSON to bounded guest specifications. This module never installs apps.
use crate::microapp;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub key: String,
    pub label: String,
    pub kind: String,
    pub required: bool,
    pub default: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub formula: String,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub from: String,
    pub to: String,
    pub guard_field: String,
    pub guard_op: String,
    pub guard_value: i32,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub initial: String,
    pub actions: Vec<Action>,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub op: String,
    pub field: String,
    pub value: String,
    pub compare: String,
    pub skip: usize,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Button {
    pub id: String,
    pub label: String,
    pub event: String,
    pub steps: Vec<Step>,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub id: String,
    pub title: String,
    pub capability: String,
    pub fields: Vec<Field>,
    pub filter_key: String,
    pub filter_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<Workflow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub behaviors: Vec<Button>,
}
impl Spec {
    pub fn checked(&self) -> Result<microapp::Spec, &'static str> {
        let mut s = microapp::Spec::new(&self.id, &self.title, &self.capability)?;
        for f in &self.fields {
            s.add(
                microapp::Field::new(&f.key, &f.label, &f.kind, f.required, &f.default)?
                    .with_formula(&f.formula)?,
            )?;
        }
        s.filter(&self.filter_key, &self.filter_label)?;
        if let Some(flow) = &self.workflow {
            s.workflow = microapp::Workflow::new(&flow.initial)?;
            for a in &flow.actions {
                s.workflow.add(microapp::Action::new(
                    &a.id,
                    &a.label,
                    &a.from,
                    &a.to,
                    &a.guard_field,
                    &a.guard_op,
                    a.guard_value,
                )?)?;
            }
        }
        for button in &self.behaviors {
            let mut b = microapp::Button::new(&button.id, &button.label, &button.event)?;
            for step in &button.steps {
                b.add(microapp::Step::new(
                    &step.op,
                    &step.field,
                    &step.value,
                    &step.compare,
                    step.skip,
                )?)?;
            }
            s.behaviors.add(b)?;
        }
        s.check()?;
        Ok(s)
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub request_id: u32,
    pub base_revision: u32,
    pub goal: String,
    pub target: Option<Spec>,
    pub app_ids: Vec<String>,
    pub record_count: usize,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub request_id: u32,
    pub base_revision: u32,
    pub decision: String,
    pub spec: Option<Spec>,
    pub reason: String,
}
pub fn request(text: &str) -> Result<Request, &'static str> {
    let r: Request = serde_json::from_str(text).map_err(|_| "invalid_app_request")?;
    if r.request_id == 0
        || r.base_revision == 0
        || r.goal.trim().is_empty()
        || microapp::Text::<128>::new(&r.goal).is_err()
        || r.app_ids.len() > microapp::APPS
        || r.record_count > microapp::ROWS
    {
        return Err("invalid_app_request");
    }
    for (i, id) in r.app_ids.iter().enumerate() {
        if !microapp::identifier(id) || r.app_ids[..i].contains(id) {
            return Err("invalid_app_catalog");
        }
    }
    if let Some(s) = &r.target {
        s.checked()?;
        if !r.app_ids.contains(&s.id) {
            return Err("invalid_app_target");
        }
    } else if r.record_count != 0 {
        return Err("invalid_app_target");
    }
    Ok(r)
}
pub fn parse(text: &str) -> Result<Plan, &'static str> {
    serde_json::from_str(text).map_err(|_| "invalid_app_plan")
}
pub fn validate(p: &Plan, r: &Request) -> Result<(), &'static str> {
    if p.request_id != r.request_id || p.base_revision != r.base_revision {
        return Err("stale_app_plan");
    }
    if p.reason.trim().is_empty() || microapp::Text::<160>::new(&p.reason).is_err() {
        return Err("invalid_app_reason");
    }
    if matches!(p.decision.as_str(), "clarify" | "decline") {
        return if p.spec.is_none() {
            Ok(())
        } else {
            Err("nonexecuting_app_spec")
        };
    }
    if p.decision != "install" {
        return Err("invalid_app_decision");
    }
    let s = p.spec.as_ref().ok_or("missing_app_spec")?.checked()?;
    if let Some(old) = &r.target {
        let old = old.checked()?;
        s.compatible(&old)?;
        if r.record_count > 0
            && s.fields()[old.fields().len()..]
                .iter()
                .any(|f| f.kind != microapp::Kind::Calculated && !f.valid(f.default.text()))
        {
            return Err("app_invalid_migration_default");
        }
    } else if r.app_ids.contains(&s.id.text().to_owned()) || r.app_ids.len() == microapp::APPS {
        return Err("app_unavailable_slot");
    }
    Ok(())
}
pub fn capabilities() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../fixtures/microapps/v1/runtime.json"))
        .expect("committed runtime catalog")
}
fn validation_failure(p: &Plan, r: &Request, code: &str) -> serde_json::Value {
    // Retain the decoded, untrusted declaration for diagnosis. This is not an
    // executable app_plan; record contents and provider configuration are absent.
    serde_json::json!({
        "event":"app_validation_failed",
        "request_id":r.request_id,
        "code":code,
        "proposal":p,
        "execution":false
    })
}
pub fn run(path: &std::path::Path, replay: Option<&std::path::Path>) -> Result<(), String> {
    let r = request(&crate::config::bounded_file(path, 16384)?)?;
    let p = if let Some(path) = replay {
        parse(&crate::config::bounded_file(path, 16384)?)?
    } else {
        let key = crate::config::load()?;
        crate::decision_source::report(crate::provider::apps(&key, &r), u64::from(r.request_id))?
            .proposal
    };
    if let Err(code) = validate(&p, &r) {
        // Stop before emitting app_plan. Diagnosis must preserve the proposal
        // that failed, rather than require another paid call to reproduce it.
        println!("{}", validation_failure(&p, &r, code));
        return Err(code.into());
    }
    println!(
        "{}",
        serde_json::json!({"event":"app_plan","source":if replay.is_some(){"replay"}else{"meta"},"plan":p,"execution":false})
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    fn behavior_spec() -> serde_json::Value {
        serde_json::json!({
            "id":"dynamic","title":"Dynamic tool","capability":"records.local",
            "fields":[
                {"key":"name","label":"Name","kind":"text","required":true,"default":"Item"},
                {"key":"amount","label":"Amount","kind":"number","required":true,"default":"0"},
                {"key":"done","label":"Done","kind":"check","required":true,"default":"false"},
                {"key":"doubled","label":"Doubled","kind":"calculated","required":true,"default":"","formula":"$amount*2"}
            ],
            "filter_key":"","filter_label":"",
            "behaviors":[{"id":"change","label":"Change","event":"press","steps":[
                {"op":"if","field":"amount","value":"0","compare":"ge","skip":1},
                {"op":"calculate","field":"amount","value":"$amount+1","compare":"","skip":0},
                {"op":"toggle","field":"done","value":"","compare":"","skip":0}
            ]}]
        })
    }
    #[test]
    fn rejected_behavior_diagnostic_retains_invalid_expression_without_app_plan() {
        let request = request(r#"{"request_id":2,"base_revision":7,"goal":"A composed operation","target":null,"app_ids":[],"record_count":0}"#).unwrap();
        let mut spec = behavior_spec();
        spec["behaviors"][0]["steps"][1]["value"] = serde_json::json!("$amount +");
        let proposal = Plan {
            request_id: 2,
            base_revision: 7,
            decision: "install".into(),
            spec: Some(serde_json::from_value(spec).unwrap()),
            reason: "This synthetic proposal must be rejected.".into(),
        };
        assert_eq!(validate(&proposal, &request), Err("formula_syntax"));
        // Exercise the actual serialization used by the diagnostic log.
        let encoded = validation_failure(&proposal, &request, "formula_syntax").to_string();
        let event: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(event["event"], "app_validation_failed");
        assert_eq!(event["request_id"], 2);
        assert_eq!(event["code"], "formula_syntax");
        assert_eq!(event["execution"], false);
        assert_eq!(event["proposal"], serde_json::to_value(&proposal).unwrap());
        assert_eq!(
            event["proposal"]["spec"]["behaviors"][0]["steps"][1]["value"],
            "$amount +"
        );
        assert!(event.get("plan").is_none());
        assert!(event.get("request").is_none());
        assert_eq!(event.as_object().unwrap().len(), 5);
    }
    #[test]
    fn behavior_json_uses_shared_validation_and_strict_fields() {
        let value = behavior_spec();
        let spec: Spec = serde_json::from_value(value.clone()).unwrap();
        spec.checked().unwrap();
        assert_eq!(serde_json::to_value(&spec).unwrap(), value);
        let invalid = [
            ("op", serde_json::json!("shell")),
            ("field", serde_json::json!("missing")),
            ("compare", serde_json::json!("unknown")),
            ("skip", serde_json::json!(6)),
        ];
        for (field, replacement) in invalid {
            let mut bad = value.clone();
            bad["behaviors"][0]["steps"][0][field] = replacement;
            assert!(
                serde_json::from_value::<Spec>(bad)
                    .unwrap()
                    .checked()
                    .is_err()
            );
        }
        let mut bad = value.clone();
        // Even a potentially skipped branch cannot write a calculated field.
        bad["behaviors"][0]["steps"][1]["field"] = serde_json::json!("doubled");
        assert!(
            serde_json::from_value::<Spec>(bad)
                .unwrap()
                .checked()
                .is_err()
        );
        let mut bad = value.clone();
        bad["behaviors"][0]["steps"][0]["field"] = serde_json::json!("name");
        assert!(
            serde_json::from_value::<Spec>(bad)
                .unwrap()
                .checked()
                .is_err()
        );
        let mut bad = value.clone();
        bad["behaviors"][0]["steps"][0]["code"] = serde_json::json!("arbitrary");
        assert!(serde_json::from_value::<Spec>(bad).is_err());
        let mut bad = value.clone();
        bad["behaviors"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<Spec>(bad).is_err());
        let mut old = value.clone();
        old.as_object_mut().unwrap().remove("behaviors");
        let spec: Spec = serde_json::from_value(old.clone()).unwrap();
        spec.checked().unwrap();
        assert!(spec.behaviors.is_empty());
        assert_eq!(serde_json::to_value(spec).unwrap(), old);
    }
    #[test]
    fn behavior_updates_are_reviewable_without_changing_existing_field_identity() {
        let value = behavior_spec();
        let original: Spec = serde_json::from_value(value.clone()).unwrap();
        let r = Request {
            request_id: 2,
            base_revision: 7,
            goal: "Change the calculation".into(),
            target: Some(original),
            app_ids: vec!["dynamic".into()],
            record_count: 2,
        };
        let mut revised = value;
        revised["behaviors"][0]["steps"][1]["value"] = serde_json::json!("$amount+2");
        let mut p = Plan {
            request_id: 2,
            base_revision: 7,
            decision: "install".into(),
            spec: Some(serde_json::from_value(revised).unwrap()),
            reason: "A reviewed replacement behavior.".into(),
        };
        validate(&p, &r).unwrap();
        p.spec.as_mut().unwrap().fields[1].key = "different".into();
        assert!(validate(&p, &r).is_err());
    }
    #[test]
    fn display_schema_is_bounded_and_updates_cannot_escalate() {
        let old:Spec=serde_json::from_value(serde_json::json!({"id":"screen","title":"Screen","capability":"display.brightness","fields":[{"key":"level","label":"Brightness %","kind":"number","required":true,"default":"50"}],"filter_key":"","filter_label":""})).unwrap();
        old.checked().unwrap();
        let mut bad = old.clone();
        bad.fields[0].default = "0".into();
        assert!(bad.checked().is_err());
        bad = old.clone();
        bad.fields[0].required = false;
        assert!(bad.checked().is_err());
        let mut record = old.clone();
        record.capability = "records.local".into();
        let r = Request {
            request_id: 1,
            base_revision: 1,
            goal: "Change app type".into(),
            target: Some(record),
            app_ids: vec!["screen".into()],
            record_count: 0,
        };
        let p = Plan {
            request_id: 1,
            base_revision: 1,
            decision: "install".into(),
            spec: Some(old),
            reason: "Proposed change".into(),
        };
        assert_eq!(validate(&p, &r), Err("app_capability_change"));
    }
    #[test]
    fn advertised_limits_match_the_guest_domain() {
        let c = capabilities();
        for cap in c["capabilities"].as_array().unwrap() {
            assert!(microapp::Capability::parse(cap.as_str().unwrap()).is_ok());
        }
        assert_eq!(c["limits"]["apps"], microapp::APPS);
        assert_eq!(c["limits"]["fields"], microapp::FIELDS);
        assert_eq!(c["limits"]["rows_per_app"], microapp::ROWS);
        for kind in c["field_kinds"].as_array().unwrap() {
            assert!(microapp::Kind::parse(kind.as_str().unwrap()).is_ok());
        }
    }
    #[test]
    fn untrusted_specs_cannot_expand_authority() {
        let r=request(r#"{"request_id":2,"base_revision":1,"goal":"An inventory","target":null,"app_ids":[],"record_count":0}"#).unwrap();
        let mut p=parse(r#"{"request_id":2,"base_revision":1,"decision":"install","spec":{"id":"stock","title":"Stock","capability":"records.local","fields":[{"key":"name","label":"Item","kind":"text","required":true,"default":""}],"filter_key":"","filter_label":""},"reason":"Local inventory."}"#).unwrap();
        validate(&p, &r).unwrap();
        p.spec.as_mut().unwrap().capability = "network".into();
        assert_eq!(validate(&p, &r), Err("app_capability_denied"));
        p.spec.as_mut().unwrap().capability = "records.local".into();
        p.base_revision = 2;
        assert_eq!(validate(&p, &r), Err("stale_app_plan"));
        p.base_revision = 1;
        p.decision = "decline".into();
        assert!(validate(&p, &r).is_err());
        p.spec = None;
        validate(&p, &r).unwrap();
        assert!(parse(r#"{"request_id":2,"base_revision":1,"decision":"install","spec":null,"reason":"x","code":"shell"}"#).is_err());
    }
}

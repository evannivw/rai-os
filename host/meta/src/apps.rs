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
}
impl Spec {
    pub fn checked(&self) -> Result<microapp::Spec, &'static str> {
        let mut s = microapp::Spec::new(&self.id, &self.title, &self.capability)?;
        for f in &self.fields {
            s.add(microapp::Field::new(
                &f.key, &f.label, &f.kind, f.required, &f.default,
            )?)?;
        }
        s.filter(&self.filter_key, &self.filter_label)?;
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
                .any(|f| !f.valid(f.default.text()))
        {
            return Err("app_invalid_migration_default");
        }
    } else if r.app_ids.contains(&s.id.text().to_owned()) || r.app_ids.len() == microapp::APPS {
        return Err("app_unavailable_slot");
    }
    Ok(())
}
pub fn run(path: &std::path::Path, replay: Option<&std::path::Path>) -> Result<(), String> {
    let r = request(&crate::config::bounded_file(path, 8192)?)?;
    let p = if let Some(path) = replay {
        parse(&crate::config::bounded_file(path, 8192)?)?
    } else {
        let key = crate::config::load()?;
        crate::decision_source::report(crate::provider::apps(&key, &r), u64::from(r.request_id))?
            .proposal
    };
    validate(&p, &r)?;
    println!(
        "{}",
        serde_json::json!({"event":"app_plan","source":if replay.is_some(){"replay"}else{"meta"},"plan":p,"execution":false})
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
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

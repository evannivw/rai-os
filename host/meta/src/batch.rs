//! Bounded plans and all-before-dispatch admission; no provider or transport I/O.
use crate::{
    domain::{self, Action, Proposal, Work},
    set_report::Snapshot,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: String,
    pub record: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub state_version: u64,
    pub decision: String,
    pub operations: Vec<Step>,
    pub reason: String,
}
pub fn parse(text: &str) -> Result<Plan, &'static str> {
    if text.len() > 2048 {
        return Err("batch_too_large");
    }
    serde_json::from_str(text).map_err(|_| "invalid_batch_json")
}
/// None is an explicit decline, never permission to substitute a fixed policy.
/// Execute must cover EXACTLY the original missing set. Validate every step before
/// returning any action, so an invalid suffix cannot partially execute a plan.
pub fn validate(
    plan: &Plan,
    snapshot: &Snapshot,
) -> Result<Option<Vec<&'static str>>, &'static str> {
    if plan.state_version != u64::from(snapshot.goal_version) {
        return Err("stale_batch");
    }
    if plan.reason.trim().is_empty()
        || plan.reason.len() > 240
        || plan.reason.chars().any(char::is_control)
    {
        return Err("invalid_batch_reason");
    }
    if plan.decision == "decline" && plan.operations.is_empty() {
        return Ok(None);
    }
    if plan.decision != "execute" {
        return Err("invalid_batch_decision");
    }
    if plan.operations.is_empty() || plan.operations.len() > 4 {
        return Err("batch_requires_1_to_4_operations");
    }
    let mut keys = Vec::new();
    for step in &plan.operations {
        let p = Proposal {
            state_version: plan.state_version,
            operation: step.operation.clone(),
            record: step.record.clone(),
            reason: plan.reason.clone(),
        };
        let Action::Index { key, .. } = domain::validate_for(&p, plan.state_version, Work::Index)?
        else {
            return Err("operation_not_allowed");
        };
        if !snapshot.missing.contains(&key) {
            return Err("batch_record_not_missing");
        }
        if keys.contains(&key) {
            return Err("duplicate_batch_record");
        }
        keys.push(key);
    }
    if keys.len() != snapshot.missing.len() {
        return Err("incomplete_batch");
    }
    Ok(Some(keys))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn snapshot() -> Snapshot {
        Snapshot::parse("goal_version=7; state=building; required=brief@1,empty@1,notes@2; ready_records=1; total_records=3; completed=1; missing=brief@1,notes@2").unwrap()
    }
    fn value() -> serde_json::Value {
        json!({"state_version":7,"decision":"execute","operations":[{"operation":"record.index","record":"notes@2"},{"operation":"record.index","record":"brief@1"}],"reason":"Complete the missing revisions."})
    }
    #[test]
    fn accepts_a_whole_plan_or_explicit_decline() {
        assert_eq!(
            validate(&parse(&value().to_string()).unwrap(), &snapshot()),
            Ok(Some(vec!["notes@2", "brief@1"]))
        );
        let mut v = value();
        v["decision"] = json!("decline");
        v["operations"] = json!([]);
        assert_eq!(
            validate(&parse(&v.to_string()).unwrap(), &snapshot()),
            Ok(None)
        );
    }
    #[test]
    fn invalid_suffix_never_returns_a_partial_plan() {
        for (path, v) in [
            ("/state_version", json!(8)),
            ("/operations/1/record", json!("notes@2")),
            ("/operations/1/record", json!("empty@1")),
            ("/operations/1/record", json!("restricted@1")),
            ("/operations/1/operation", json!("shell")),
            ("/operations/1/operation", json!("record.count")),
            ("/reason", json!("bad\nreason")),
            ("/decision", json!("decline")),
        ] {
            let mut p = value();
            *p.pointer_mut(path).unwrap() = v;
            assert!(validate(&parse(&p.to_string()).unwrap(), &snapshot()).is_err());
        }
        let mut v = value();
        v["operations"].as_array_mut().unwrap().pop();
        assert_eq!(
            validate(&parse(&v.to_string()).unwrap(), &snapshot()),
            Err("incomplete_batch")
        );
        v["operations"] = json!([]);
        assert!(validate(&parse(&v.to_string()).unwrap(), &snapshot()).is_err());
        v["operations"] = json!([{}, {}, {}, {}, {}]);
        assert!(parse(&v.to_string()).is_err());
    }
    #[test]
    fn rejects_extra_duplicate_fields_and_oversize_input() {
        let text = value().to_string();
        assert!(parse(&text.replacen('{', "{\"state_version\":7,", 1)).is_err());
        let mut v = value();
        v["operations"][0]["command"] = json!("info");
        assert!(parse(&v.to_string()).is_err());
        assert!(parse(&" ".repeat(2049)).is_err());
    }
}

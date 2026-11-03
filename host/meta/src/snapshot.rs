//! Validate saved declarative state with the same domain as a fresh guest.
use crate::{
    apps::Spec,
    microapp::{APPS, FIELDS, ROWS, Runtime, Text},
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredApp {
    spec: Spec,
    rows: Vec<Vec<String>>,
    #[serde(default)]
    states: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u32,
    apps: Vec<StoredApp>,
}
fn validate(text: &str) -> Result<usize, &'static str> {
    let s: Snapshot = serde_json::from_str(text).map_err(|_| "invalid_snapshot")?;
    if s.version != 1 || s.apps.len() > APPS {
        return Err("unsupported_snapshot");
    }
    let mut runtime = Runtime::new();
    for a in &s.apps {
        let spec = a.spec.checked()?;
        let index = runtime.install(&spec, runtime.revision(), None)?;
        if a.rows.len() > ROWS {
            return Err("app_row_limit");
        }
        if (spec.workflow.enabled() && a.states.len() != a.rows.len())
            || (!spec.workflow.enabled() && !a.states.is_empty())
        {
            return Err("snapshot_workflow_states");
        }
        for (rowno, row) in a.rows.iter().enumerate() {
            if row.len() != spec.fields().len() {
                return Err("invalid_snapshot_columns");
            }
            let mut values = [Text::EMPTY; FIELDS];
            for (i, v) in row.iter().enumerate() {
                values[i] = Text::new(v)?;
            }
            let normalized = spec.materialize(&values)?;
            if normalized != values {
                return Err("snapshot_derived_mismatch");
            }
            runtime.save(index, None, &values)?;
            if spec.workflow.enabled() {
                runtime.restore_state(index, rowno, &a.states[rowno])?;
            }
        }
    }
    Ok(s.apps.len())
}
pub fn run(path: &std::path::Path) -> Result<(), String> {
    let text = crate::config::bounded_file(path, 32768)?;
    let count = validate(&text)?;
    println!(
        "{}",
        serde_json::json!({"event":"snapshot_valid","apps":count})
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workflow_snapshot_requires_complete_known_states() {
        let mut s = serde_json::json!({"version":1,"apps":[{"spec":{"id":"jobs","title":"Jobs","capability":"records.local","fields":[{"key":"name","label":"Name","kind":"text","required":true,"default":""}],"filter_key":"","filter_label":"","workflow":{"initial":"new","actions":[{"id":"close","label":"Close","from":"new","to":"closed","guard_field":"","guard_op":"none","guard_value":0}]}},"rows":[["Job A"]],"states":["closed"]}]});
        assert_eq!(validate(&s.to_string()), Ok(1));
        s["apps"][0]["states"] = serde_json::json!([]);
        assert_eq!(validate(&s.to_string()), Err("snapshot_workflow_states"));
        s["apps"][0]["states"] = serde_json::json!(["forged"]);
        assert_eq!(validate(&s.to_string()), Err("workflow_bad_state"));
    }
    #[test]
    fn derived_snapshot_values_must_match_recomputation() {
        let mut s = serde_json::json!({"version":1,"apps":[{"spec":{"id":"custom","title":"Custom","capability":"records.local","fields":[{"key":"x","label":"Input","kind":"number","required":true,"default":"1"},{"key":"y","label":"Result","kind":"calculated","required":true,"default":"","formula":"$x*2"}],"filter_key":"","filter_label":""},"rows":[["3","6"]]}]});
        assert_eq!(validate(&s.to_string()), Ok(1));
        s["apps"][0]["rows"][0][1] = "999".into();
        assert_eq!(validate(&s.to_string()), Err("snapshot_derived_mismatch"));
    }
    #[test]
    fn restoration_rejects_unknown_fields_duplicates_types_and_limits() {
        let mut s = serde_json::json!({"version":1,"apps":[{"spec":{"id":"stock","title":"Stock","capability":"records.local","fields":[{"key":"qty","label":"Quantity","kind":"number","required":true,"default":"1"}],"filter_key":"","filter_label":""},"rows":[["12"]]}]});
        assert_eq!(validate(&s.to_string()), Ok(1));
        s["apps"][0]["rows"][0][0] = "bad".into();
        assert!(validate(&s.to_string()).is_err());
        s["apps"][0]["rows"][0][0] = "12".into();
        s["apps"][0]["rows"][0] = serde_json::json!(["12", "extra"]);
        assert!(validate(&s.to_string()).is_err());
        s["apps"][0]["rows"] = serde_json::json!([["12"]]);
        let a = s["apps"][0].clone();
        s["apps"].as_array_mut().unwrap().push(a);
        assert!(validate(&s.to_string()).is_err());
        s["apps"].as_array_mut().unwrap().pop();
        s["apps"][0]["spec"]["network"] = true.into();
        assert!(validate(&s.to_string()).is_err());
        assert!(validate(r#"{"version":1,"version":1,"apps":[]}"#).is_err());
    }
}

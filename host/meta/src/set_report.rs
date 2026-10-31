//! Pure decoding of guest-owned requirements/readiness and bounded model context.
pub use crate::domain::IndexSet as Snapshot;
use crate::domain::{DecisionContext, RECORDS, Work};
use serde::Deserialize;
use std::collections::BTreeMap;

pub fn keys<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<Vec<&'static str>, &'static str> {
    let mut result = Vec::new();
    for value in values {
        let key = RECORDS
            .iter()
            .find(|(key, _)| *key == value)
            .ok_or("record_not_allowed")?
            .0;
        if result
            .iter()
            .any(|old: &&str| old.split('@').next() == key.split('@').next())
        {
            return Err("duplicate_or_conflicting_requirement");
        }
        result.push(key);
    }
    if result.is_empty() || result.len() > 4 {
        return Err("set_requires_1_to_4_records");
    }
    result.sort_unstable();
    Ok(result)
}

impl Snapshot {
    pub fn parse(payload: &str) -> Result<Self, &'static str> {
        if payload.len() > 512 {
            return Err("invalid_set_status");
        }
        let mut fields = BTreeMap::new();
        for pair in payload.split("; ") {
            let (k, v) = pair.split_once('=').ok_or("invalid_set_status")?;
            if fields.insert(k, v).is_some() {
                return Err("duplicate_set_field");
            }
        }
        let get = |k| fields.get(k).copied().ok_or("missing_set_field");
        let number = |k| -> Result<u64, &'static str> {
            let text = get(k)?;
            if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
                return Err("invalid_set_number");
            }
            text.parse().map_err(|_| "invalid_set_number")
        };
        if fields.len() != 7 {
            return Err("unexpected_set_fields");
        }
        let required = keys(get("required")?.split(','))?;
        let missing = if get("missing")? == "-" {
            Vec::new()
        } else {
            keys(get("missing")?.split(','))?
        };
        let version = u32::try_from(number("goal_version")?).map_err(|_| "invalid_set_version")?;
        let completed = number("completed")?;
        if version == 0
            || completed > 6
            || required.join(",") != get("required")?
            || (!missing.is_empty() && missing.join(",") != get("missing")?)
            || missing.iter().any(|k| !required.contains(k))
            || number("total_records")? != required.len() as u64
            || number("ready_records")? != (required.len() - missing.len()) as u64
            || number("ready_records")? > completed
            || get("state")?
                != if missing.is_empty() {
                    "complete"
                } else {
                    "building"
                }
        {
            return Err("inconsistent_set_status");
        }
        Ok(Self {
            goal_version: version,
            required,
            missing,
            completed,
        })
    }
    pub fn context(&self) -> Result<DecisionContext, &'static str> {
        if self.missing.is_empty() {
            return Err("set_already_complete");
        }
        Ok(DecisionContext {
            operation: Work::Index,
            state_version: self.goal_version.into(),
            authorized_records: self
                .missing
                .iter()
                .map(|key| *RECORDS.iter().find(|(k, _)| k == key).unwrap())
                .collect(),
            completed_results: Vec::new(),
            required_record: None,
            index_set: Some(self.clone()),
        })
    }
    /// Structural gate only. Independent Python checks exact source bytes/indexes.
    pub fn check_artifact(&self, payload: &str) -> Result<(), &'static str> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Artifact {
            format: String,
            goal_version: u32,
            required: Vec<String>,
            records: Vec<Row>,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Row {
            record: String,
            bytes: u64,
            source_hex: String,
            index: String,
        }
        if !self.missing.is_empty() || payload.len() > 4096 || !payload.is_ascii() {
            return Err("invalid_set_artifact");
        }
        let a: Artifact = serde_json::from_str(payload).map_err(|_| "invalid_set_artifact")?;
        if a.format != "rai-index-set-v1"
            || a.goal_version != self.goal_version
            || a.required != self.required
            || a.records.len() != self.required.len()
        {
            return Err("mismatched_set_artifact");
        }
        for (row, key) in a.records.iter().zip(&self.required) {
            let bytes = RECORDS.iter().find(|(k, _)| k == key).unwrap().1;
            if row.record != *key
                || row.bytes != bytes
                || row.source_hex.len() != bytes as usize * 2
                || !row
                    .source_hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || row.index.is_empty()
                || row.index.len() > 384
                || row.index.bytes().any(|b| b.is_ascii_control())
            {
                return Err("mismatched_set_record");
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const STATUS: &str = "goal_version=7; state=building; required=brief@1,notes@2; ready_records=1; total_records=2; completed=2; missing=notes@2";
    #[test]
    fn context_exposes_full_current_set_but_authorizes_only_missing() {
        let s = Snapshot::parse(STATUS).unwrap();
        let c = s.context().unwrap();
        assert_eq!(c.state_version, 7);
        assert_eq!(c.authorized_records, vec![("notes@2", 44)]);
        assert_eq!(c.index_set.unwrap(), s);
        for bad in [
            STATUS.replace("7;", "0;"),
            STATUS.replace("ready_records=1", "ready_records=2"),
            STATUS.replace("missing=notes@2", "missing=notes@1"),
            STATUS.replace("completed=2", "completed=0"),
            format!("{STATUS}; goal_version=7"),
        ] {
            assert!(Snapshot::parse(&bad).is_err());
        }
        for values in [
            vec![],
            vec!["notes@1", "notes@2"],
            vec!["brief@1", "brief@1"],
            vec!["restricted@1"],
        ] {
            assert!(keys(values).is_err());
        }
    }
}

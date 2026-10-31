//! Proposal validation and observed-result checks. No network, secrets or processes.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STATE_VERSION: u64 = 1;
pub const RECORDS: [(&str, u64); 6] = [
    ("brief@1", 56),
    ("notes@1", 30),
    ("notes@2", 44),
    ("checklist@1", 41),
    ("empty@1", 0),
    ("appendix@1", 108),
];
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub state_version: u64,
    pub operation: String,
    pub record: String,
    pub reason: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Work {
    #[serde(rename = "record.count")]
    Count,
    #[serde(rename = "record.index")]
    Index,
}
impl Work {
    pub fn name(self) -> &'static str {
        match self {
            Self::Count => "record.count",
            Self::Index => "record.index",
        }
    }
    pub fn is_count(&self) -> bool {
        *self == Self::Count
    }
}
pub enum Action {
    Index {
        key: &'static str,
        bytes: u64,
    },
    Count {
        key: &'static str,
        id: u64,
        bytes: u64,
    },
    Decline,
}
pub fn validate_goal(goal: &str) -> Result<(), &'static str> {
    if goal.trim().is_empty() || goal.len() > 512 || goal.chars().any(char::is_control) {
        return Err("goal_requires_1_to_512_bytes_without_control_characters");
    }
    Ok(())
}
pub fn parse_proposal(text: &str) -> Result<Proposal, &'static str> {
    if text.len() > 2048 {
        return Err("proposal_too_large");
    }
    serde_json::from_str(text).map_err(|_| "invalid_proposal_json")
}
pub fn validate(proposal: &Proposal) -> Result<Action, &'static str> {
    validate_at(proposal, STATE_VERSION)
}
pub fn validate_at(proposal: &Proposal, version: u64) -> Result<Action, &'static str> {
    validate_for(proposal, version, Work::Count)
}
pub fn validate_for(proposal: &Proposal, version: u64, work: Work) -> Result<Action, &'static str> {
    if proposal.state_version != version {
        return Err("stale_proposal");
    }
    if proposal.reason.trim().is_empty()
        || proposal.reason.len() > 240
        || proposal.reason.chars().any(char::is_control)
    {
        return Err("invalid_proposal_reason");
    }
    match proposal.operation.as_str() {
        "decline" if proposal.record.is_empty() => Ok(Action::Decline),
        operation if operation == work.name() => {
            let (index, &(key, bytes)) = RECORDS
                .iter()
                .enumerate()
                .find(|(_, (key, _))| *key == proposal.record)
                .ok_or("record_not_allowed")?;
            Ok(match work {
                Work::Count => Action::Count {
                    key,
                    id: index as u64 + 1,
                    bytes,
                },
                Work::Index => Action::Index { key, bytes },
            })
        }
        _ => Err("operation_not_allowed"),
    }
}
/// Only validated actions reach this closure; rejected proposals execute nothing.
pub fn execute<T>(
    proposal: &Proposal,
    run: impl FnOnce(&Action) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let action = validate(proposal)?;
    match action {
        Action::Decline => Ok(None),
        _ => run(&action).map(Some),
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Observed {
    pub record: &'static str,
    pub bytes: u64,
    pub tokens: u64,
}
pub fn observe(action: &Action, payload: &str) -> Result<Observed, &'static str> {
    let Action::Count { key, id, bytes } = action else {
        return Err("no_guest_action");
    };
    let mut fields = BTreeMap::new();
    for pair in payload.split("; ") {
        let (name, value) = pair.split_once('=').ok_or("invalid_guest_result")?;
        if fields.insert(name, value).is_some() {
            return Err("duplicate_guest_field");
        }
    }
    let get = |name| fields.get(name).copied().ok_or("missing_guest_field");
    let number = |name| {
        get(name)?
            .parse::<u64>()
            .map_err(|_| "invalid_guest_number")
    };
    let count = number("tokens")?;
    if get("record")? != *key
        || number("bytes")? != *bytes
        || count > *bytes
        || get("worker")? != "EL0"
        || get("a_state")? != "exited"
        || get("b_state")? != "exited"
        || number("a_code")? != 0
        || number("b_code")? != 0
    {
        return Err("unexpected_guest_result");
    }
    let packed = (id << 32) | (bytes << 16) | count;
    if number("a_value")? != packed || number("b_value")? != packed {
        return Err("mismatched_guest_result");
    }
    // Shape/completion checks only. The separate Python verifier checks the count
    // against immutable source bytes; this adapter does not declare AI correctness.
    Ok(Observed {
        record: key,
        bytes: *bytes,
        tokens: count,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn index_permission_is_explicit_and_does_not_expand_legacy_modes() {
        let mut p = proposal();
        p.operation = "record.index".into();
        assert!(validate(&p).is_err());
        assert!(matches!(
            validate_for(&p, 1, Work::Index).unwrap(),
            Action::Index { key: "notes@2", .. }
        ));
        assert!(validate_for(&p, 2, Work::Index).is_err());
        p.operation = "record.count".into();
        assert!(validate_for(&p, 1, Work::Index).is_err());
        p.operation = "record.index".into();
        p.record = "restricted@1".into();
        assert!(validate_for(&p, 1, Work::Index).is_err());
    }
    fn proposal() -> Proposal {
        Proposal {
            state_version: 1,
            operation: "record.count".into(),
            record: "notes@2".into(),
            reason: "Use the latest notes revision.".into(),
        }
    }
    #[test]
    fn rejected_proposals_never_dispatch() {
        for (operation, record, version) in [
            ("shell", "notes@2", 1),
            ("record.count", "restricted@1", 1),
            ("record.count", "notes@3", 1),
            ("record.count", "notes@2", 0),
            ("record.count", "notes@2\tinfo", 1),
            ("decline", "notes@2", 1),
        ] {
            let p = Proposal {
                operation: operation.into(),
                record: record.into(),
                state_version: version,
                ..proposal()
            };
            assert!(execute::<()>(&p, |_| panic!("Rejected proposal executed")).is_err());
        }
        let p = Proposal {
            operation: "decline".into(),
            record: String::new(),
            ..proposal()
        };
        assert!(
            execute::<()>(&p, |_| panic!("Decline executed"))
                .unwrap()
                .is_none()
        );
        let mut calls = 0;
        execute(&proposal(), |action| {
            calls += 1;
            assert!(matches!(action, Action::Count { key: "notes@2", .. }));
            Ok(())
        })
        .unwrap();
        assert_eq!(calls, 1);
    }
    #[test]
    fn json_cannot_smuggle_extra_fields_duplicates_or_text() {
        let valid = serde_json::to_string(&proposal()).unwrap();
        assert!(parse_proposal(&valid).is_ok());
        for text in [
            format!("```json\n{valid}\n```"),
            valid.replace("{", "{\"shell\":\"rm\","),
            valid.replace("{", "{\"state_version\":1,"),
            "x".repeat(2049),
        ] {
            assert!(parse_proposal(&text).is_err());
        }
    }
    #[test]
    fn input_limits_reject_empty_large_and_control_text() {
        for goal in [
            "".to_string(),
            " ".to_string(),
            "x".repeat(513),
            "x\ninfo".into(),
        ] {
            assert!(validate_goal(&goal).is_err());
        }
        assert!(validate_goal("Cuenta las palabras de la revisión más reciente.").is_ok());
        for reason in ["".to_string(), "x".repeat(241), "\x1b[2J".into()] {
            assert!(
                validate(&Proposal {
                    reason,
                    ..proposal()
                })
                .is_err()
            );
        }
    }
    #[test]
    fn completion_checks_reject_partial_mismatched_and_duplicate_results() {
        let action = validate(&proposal()).unwrap();
        let packed = (3_u64 << 32) | (44 << 16) | 9;
        let good = format!(
            "record=notes@2; bytes=44; tokens=9; worker=EL0; a_state=exited; b_state=exited; a_code=0; b_code=0; a_value={packed}; b_value={packed}"
        );
        assert_eq!(observe(&action, &good).unwrap().tokens, 9);
        for bad in [
            good.replace("notes@2", "notes@1"),
            good.replace("tokens=9", "tokens=45"),
            good.replace("a_code=0", "a_code=2"),
            good.replace("b_state=exited", "b_state=faulted"),
            good.replace("bytes=44", "bytes=43"),
            format!("{good}; tokens=9"),
            "record=notes@2".into(),
        ] {
            assert!(observe(&action, &bad).is_err());
        }
    }
}

/// A bounded, already-observed snapshot; never includes future scenario events.
#[derive(Clone, Serialize)]
pub struct DecisionContext {
    #[serde(skip_serializing_if = "Work::is_count")]
    pub operation: Work,
    pub state_version: u64,
    pub authorized_records: Vec<(&'static str, u64)>,
    pub completed_results: Vec<Observed>,
    pub required_record: Option<&'static str>,
}
impl DecisionContext {
    pub fn single() -> Self {
        Self {
            operation: Work::Count,
            state_version: STATE_VERSION,
            authorized_records: RECORDS.to_vec(),
            completed_results: Vec::new(),
            required_record: None,
        }
    }
}

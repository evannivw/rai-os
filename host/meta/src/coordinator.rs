//! Pure bounded state: released revisions, observed results and admission policy.
use crate::domain::{self, Action, DecisionContext, Observed, Proposal};
use serde::Deserialize;
pub const GOAL: &str = "Maintain a token count for the latest released revision of notes. Reuse observed results for unchanged revisions.";
pub const MAX_CALLS: usize = 4;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub at_ms: u64,
    pub record: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub events: Vec<Release>,
}
impl Scenario {
    pub fn parse(text: &str) -> Result<Self, &'static str> {
        let scenario: Self = serde_json::from_str(text).map_err(|_| "invalid_scenario")?;
        if scenario.events.is_empty() || scenario.events.len() > 8 {
            return Err("scenario_requires_1_to_8_events");
        }
        let mut previous = 0;
        for event in &scenario.events {
            revision(&event.record)?;
            if event.at_ms < previous || event.at_ms > 30_000 {
                return Err("invalid_event_time");
            }
            previous = event.at_ms;
        }
        Ok(scenario)
    }
}
fn revision(key: &str) -> Result<usize, &'static str> {
    match key {
        "notes@1" => Ok(1),
        "notes@2" => Ok(2),
        _ => Err("unsupported_release"),
    }
}
#[derive(Default)]
pub struct State {
    version: u64,
    latest: usize,
    released: [bool; 2],
    completed: Vec<Observed>,
}
impl State {
    pub fn version(&self) -> u64 {
        self.version
    }
    pub fn release(&mut self, key: &str) -> Result<bool, &'static str> {
        let rev = revision(key)?;
        // Duplicate/older events cannot roll back the goal or invalidate a result.
        if rev <= self.latest {
            return Ok(false);
        }
        self.latest = rev;
        self.released[rev - 1] = true;
        self.version += 1;
        Ok(true)
    }
    pub fn pending(&self) -> Option<&'static str> {
        let key = match self.latest {
            1 => "notes@1",
            2 => "notes@2",
            _ => return None,
        };
        (!self.completed.iter().any(|r| r.record == key)).then_some(key)
    }
    pub fn snapshot(&self) -> DecisionContext {
        DecisionContext {
            operation: crate::domain::Work::Count,
            state_version: self.version,
            authorized_records: domain::RECORDS
                .iter()
                .copied()
                .filter(|(key, _)| revision(key).is_ok_and(|rev| self.released[rev - 1]))
                .collect(),
            completed_results: self.completed.clone(),
            index_set: None,
            pilot: None,
            required_record: match self.latest {
                1 => Some("notes@1"),
                2 => Some("notes@2"),
                _ => None,
            },
        }
    }
    pub fn admit(&self, proposal: &Proposal) -> Result<Action, &'static str> {
        let action = domain::validate_at(proposal, self.version)?;
        if let Action::Count { key, .. } = &action
            && self.pending() != Some(*key)
        {
            return Err("record_not_pending");
        }
        Ok(action)
    }
    /// A legally dispatched operation may finish after a newer release. Retain
    /// its immutable result, but it never fulfills the newer revision's goal.
    pub fn complete(&mut self, observed: Observed) -> Result<(), &'static str> {
        let rev = revision(observed.record)?;
        if !self.released[rev - 1]
            || self.completed.iter().any(|r| r.record == observed.record)
            || !domain::RECORDS.contains(&(observed.record, observed.bytes))
            || observed.tokens > observed.bytes
        {
            return Err("invalid_or_duplicate_observation");
        }
        self.completed.push(observed);
        self.version += 1;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn proposal(version: u64, key: &str) -> Proposal {
        Proposal {
            state_version: version,
            operation: "record.count".into(),
            record: key.into(),
            reason: "Process the current revision.".into(),
        }
    }
    fn result(key: &'static str) -> Observed {
        Observed {
            record: key,
            bytes: if key == "notes@1" { 30 } else { 44 },
            tokens: if key == "notes@1" { 7 } else { 9 },
        }
    }
    #[test]
    fn snapshot_hides_future_and_carries_actual_observations() {
        let mut state = State::default();
        state.release("notes@1").unwrap();
        let snap = state.snapshot();
        assert_eq!(snap.authorized_records, vec![("notes@1", 30)]);
        assert!(snap.completed_results.is_empty());
        state.complete(result("notes@1")).unwrap();
        assert_eq!(state.pending(), None);
        state.release("notes@2").unwrap();
        let snap = state.snapshot();
        assert_eq!(snap.state_version, 3);
        assert_eq!(snap.completed_results, vec![result("notes@1")]);
        assert_eq!(snap.required_record, Some("notes@2"));
    }
    #[test]
    fn stale_future_completed_and_old_revision_proposals_are_denied() {
        let mut state = State::default();
        state.release("notes@1").unwrap();
        assert!(state.admit(&proposal(1, "notes@2")).is_err());
        state.release("notes@2").unwrap();
        assert!(state.admit(&proposal(1, "notes@1")).is_err());
        assert!(state.admit(&proposal(2, "notes@1")).is_err());
        assert!(state.admit(&proposal(2, "notes@2")).is_ok());
        state.complete(result("notes@2")).unwrap();
        assert!(state.admit(&proposal(3, "notes@2")).is_err());
    }
    #[test]
    fn duplicate_and_old_events_do_not_change_state_or_repeat_work() {
        let mut state = State::default();
        state.release("notes@2").unwrap();
        state.complete(result("notes@2")).unwrap();
        let before = state.version();
        assert!(!state.release("notes@2").unwrap());
        assert!(!state.release("notes@1").unwrap());
        assert_eq!(state.version(), before);
        assert!(state.pending().is_none());
        assert!(state.complete(result("notes@2")).is_err());
    }
    #[test]
    fn superseded_result_never_fulfills_new_goal() {
        let mut state = State::default();
        state.release("notes@1").unwrap();
        assert!(state.admit(&proposal(1, "notes@1")).is_ok());
        state.release("notes@2").unwrap();
        state.complete(result("notes@1")).unwrap();
        assert_eq!(state.pending(), Some("notes@2"));
        assert_eq!(state.version(), 3);
    }
    #[test]
    fn scenario_limits_reject_unknown_out_of_order_and_unbounded_events() {
        for text in [
            r#"{"events":[]}"#,
            r#"{"events":[{"at_ms":30001,"record":"notes@1"}]}"#,
            r#"{"events":[{"at_ms":0,"record":"restricted@1"}]}"#,
            r#"{"events":[{"at_ms":2,"record":"notes@1"},{"at_ms":1,"record":"notes@2"}]}"#,
        ] {
            assert!(Scenario::parse(text).is_err());
        }
    }
}

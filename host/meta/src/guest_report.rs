//! Decode bounded guest coordinator observations; no host-owned goal/admission state.
use crate::domain::{self, Action, DecisionContext};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Snapshot {
    pub state: String,
    pub event: String,
    pub ticket: u32,
    pub record: Option<&'static str>,
    pub bytes: u64,
    pub tokens: u64,
    pub completed: u64,
    pub a_calls: u8,
    pub b_calls: u8,
    pub a_slices: u8,
    pub b_slices: u8,
    pub trace: String,
}
impl Snapshot {
    pub fn parse(payload: &str) -> Result<Self, &'static str> {
        if payload.len() > 2048 {
            return Err("invalid_guest_snapshot");
        }
        let mut fields = BTreeMap::new();
        for pair in payload.split("; ") {
            let (name, value) = pair.split_once('=').ok_or("invalid_guest_snapshot")?;
            if fields.insert(name, value).is_some() {
                return Err("duplicate_guest_field");
            }
        }
        if fields.len() != 14 {
            return Err("unexpected_guest_fields");
        }
        let get = |name| fields.get(name).copied().ok_or("missing_guest_field");
        let number = |name| -> Result<u64, &'static str> {
            let value = get(name)?;
            if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err("invalid_guest_number");
            }
            value.parse().map_err(|_| "invalid_guest_number")
        };
        if get("coordinator")? != "EL0" || get("worker")? != "EL0" {
            return Err("unexpected_guest_runtime");
        }
        let raw_key = get("record")?;
        let record = if raw_key == "none" {
            None
        } else {
            Some(
                domain::RECORDS
                    .iter()
                    .find(|(key, _)| *key == raw_key)
                    .ok_or("unknown_guest_record")?
                    .0,
            )
        };
        let snapshot = Self {
            state: get("state")?.into(),
            event: get("event")?.into(),
            ticket: u32::try_from(number("ticket")?).map_err(|_| "invalid_guest_ticket")?,
            record,
            bytes: number("bytes")?,
            tokens: number("tokens")?,
            completed: number("completed")?,
            a_calls: u8::try_from(number("a_calls")?).map_err(|_| "invalid_guest_budget")?,
            b_calls: u8::try_from(number("b_calls")?).map_err(|_| "invalid_guest_budget")?,
            a_slices: u8::try_from(number("a_slices")?).map_err(|_| "invalid_guest_budget")?,
            b_slices: u8::try_from(number("b_slices")?).map_err(|_| "invalid_guest_budget")?,
            trace: get("trace")?.into(),
        };
        if snapshot.event.is_empty()
            || snapshot.event.len() > 64
            || !snapshot
                .event
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
            || snapshot.a_calls > 16
            || snapshot.b_calls > 16
            || snapshot.a_slices > 6
            || snapshot.b_slices > 6
            || snapshot.completed > u64::from(u32::MAX)
        {
            return Err("invalid_guest_snapshot");
        }
        let events: Vec<_> = snapshot
            .trace
            .split(',')
            .filter(|s| !s.is_empty())
            .collect();
        if events.len() > 96
            || (!snapshot.trace.is_empty() && events.join(",") != snapshot.trace)
            || events.iter().any(|e| {
                e.len() != 2
                    || !matches!(e.as_bytes()[0], b'A' | b'B')
                    || !b"SRDWHOPFL".contains(&e.as_bytes()[1])
            })
        {
            return Err("invalid_guest_trace");
        }
        match snapshot.state.as_str() {
            "stopped" | "idle"
                if snapshot.record.is_none()
                    && snapshot.ticket == 0
                    && snapshot.bytes == 0
                    && snapshot.tokens == 0
                    && snapshot.completed == 0 => {}
            "awaiting_proposal"
                if snapshot.record.is_some()
                    && snapshot.ticket != 0
                    && snapshot.bytes == 0
                    && snapshot.tokens == 0 => {}
            "completed"
                if snapshot.record.is_some()
                    && snapshot.ticket != 0
                    && snapshot.completed != 0
                    && domain::RECORDS.contains(&(snapshot.record.unwrap(), snapshot.bytes))
                    && snapshot.tokens <= snapshot.bytes => {}
            "failed" => return Err("guest_session_failed"),
            _ => return Err("inconsistent_guest_snapshot"),
        }
        Ok(snapshot)
    }
    pub fn context(&self) -> Result<DecisionContext, &'static str> {
        if self.state != "awaiting_proposal" {
            return Err("guest_not_requesting_proposal");
        }
        let record = self.record.ok_or("missing_guest_goal")?;
        let metadata = *domain::RECORDS
            .iter()
            .find(|(key, _)| *key == record)
            .ok_or("unknown_guest_record")?;
        Ok(DecisionContext {
            state_version: u64::from(self.ticket),
            authorized_records: vec![metadata],
            completed_results: Vec::new(),
            required_record: Some(record),
        })
    }
    pub fn same_goal_result(&self, other: &Self) -> bool {
        self.state == other.state
            && self.ticket == other.ticket
            && self.record == other.record
            && self.bytes == other.bytes
            && self.tokens == other.tokens
            && self.completed == other.completed
    }
    pub fn check_observation(&self, before: &Self, key: &str) -> Result<(), &'static str> {
        let duplicate = before.record == Some(key);
        if self.b_calls != 0
            || self.b_slices != 0
            || self.a_calls != 2
            || self.completed != before.completed
            || (duplicate && (self.event != "duplicate" || !self.same_goal_result(before)))
            || (!duplicate
                && (self.event != "observed"
                    || self.state != "awaiting_proposal"
                    || self.record != Some(key)
                    || self.ticket <= before.ticket))
        {
            return Err("unexpected_guest_observation");
        }
        Ok(())
    }
    pub fn check_completion(
        &self,
        before: &Self,
        action: &Action,
        ticket: u32,
    ) -> Result<(), &'static str> {
        let Action::Count { key, bytes, .. } = action else {
            return Err("no_guest_action");
        };
        if before.state != "awaiting_proposal"
            || before.ticket != ticket
            || before.record != Some(key)
            || self.state != "completed"
            || self.event != "executed"
            || self.ticket != ticket
            || self.record != Some(key)
            || self.bytes != *bytes
            || before.completed.checked_add(1) != Some(self.completed)
            || self.a_calls != 5
            || self.b_calls != 3
        {
            return Err("unexpected_guest_completion");
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const WAITING: &str = "coordinator=EL0; worker=EL0; state=awaiting_proposal; event=observed; ticket=3; record=notes@2; bytes=0; tokens=0; completed=1; a_calls=2; b_calls=0; a_slices=0; b_slices=0; trace=AO,AH";
    #[test]
    fn context_comes_only_from_the_guests_pending_ticket_and_record() {
        let state = Snapshot::parse(WAITING).unwrap();
        let context = state.context().unwrap();
        assert_eq!(context.state_version, 3);
        assert_eq!(context.authorized_records, vec![("notes@2", 44)]);
        assert_eq!(context.required_record, Some("notes@2"));
        assert!(context.completed_results.is_empty()); // No invented/cached history.
    }
    #[test]
    fn malformed_or_inconsistent_guest_reports_are_not_model_context() {
        for bad in [
            format!("{WAITING}; ticket=4"),
            WAITING.replace("ticket=3", "ticket=4294967296"),
            WAITING.replace("ticket=3", "ticket=+3"),
            WAITING.replace("notes@2", "restricted@1"),
            WAITING.replace("bytes=0", "bytes=44"),
            WAITING.replace("AO,AH", "AO,,AH"),
            WAITING.replace("b_calls=0", "b_calls=17"),
            WAITING.replace("state=awaiting_proposal", "state=failed"),
        ] {
            assert!(Snapshot::parse(&bad).is_err());
        }
    }
    #[test]
    fn success_requires_matching_goal_and_observed_counter_advance() {
        let before = Snapshot::parse(WAITING).unwrap();
        let good = WAITING
            .replace("awaiting_proposal", "completed")
            .replace("observed", "executed")
            .replace("bytes=0", "bytes=44")
            .replace("tokens=0", "tokens=9")
            .replace("completed=1", "completed=2")
            .replace("a_calls=2", "a_calls=5")
            .replace("b_calls=0", "b_calls=3");
        let action = Action::Count {
            key: "notes@2",
            id: 3,
            bytes: 44,
        };
        Snapshot::parse(&good)
            .unwrap()
            .check_completion(&before, &action, 3)
            .unwrap();
        for bad in [
            good.replace("completed=2", "completed=1"),
            good.replace("ticket=3", "ticket=2"),
            good.replace("b_calls=3", "b_calls=0"),
            good.replace("event=executed", "event=duplicate"),
        ] {
            assert!(
                Snapshot::parse(&bad)
                    .unwrap()
                    .check_completion(&before, &action, 3)
                    .is_err()
            );
        }
    }
}

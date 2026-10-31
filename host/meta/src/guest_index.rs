//! Validate observed index artifacts and reuse provenance, without computing source results.
use crate::guest_report::Snapshot;
use std::collections::BTreeMap;

pub fn validate(payload: &str, snapshot: &Snapshot) -> Result<(), &'static str> {
    let index = snapshot.index.as_ref().ok_or("guest_not_indexing")?;
    if payload.len() > 512 || snapshot.state != "completed" {
        return Err("invalid_guest_artifact");
    }
    let mut fields = BTreeMap::new();
    for pair in payload.split("; ") {
        let (key, value) = pair.split_once('=').ok_or("invalid_guest_artifact")?;
        if fields.insert(key, value).is_some() {
            return Err("duplicate_guest_field");
        }
    }
    let get = |name| fields.get(name).copied().ok_or("missing_guest_field");
    if fields.len() != 5
        || get("record")? != snapshot.record.ok_or("missing_guest_goal")?
        || get("bytes")? != snapshot.bytes.to_string()
        || get("worker")? != "EL0"
        || get("state")? != "completed"
    {
        return Err("mismatched_guest_artifact");
    }
    let text = get("index")?;
    if text.len() as u64 != index.output_bytes || text.is_empty() || text.len() > 384 {
        return Err("invalid_guest_index_length");
    }
    if text == "-" {
        return if snapshot.tokens == 0 {
            Ok(())
        } else {
            Err("invalid_guest_index_tokens")
        };
    }
    let mut previous = "";
    let mut total = 0_u64;
    for entry in text.split(',') {
        let (word, number) = entry.split_once(':').ok_or("invalid_guest_index_format")?;
        if word.is_empty()
            || word.len() > 128
            || word <= previous
            || !word
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            || number.starts_with('0')
            || !number.bytes().all(|b| b.is_ascii_digit())
        {
            return Err("invalid_guest_index_format");
        }
        let count = number
            .parse::<u64>()
            .ok()
            .filter(|n| (1..=64).contains(n))
            .ok_or("invalid_guest_index_format")?;
        total = total
            .checked_add(count)
            .ok_or("invalid_guest_index_tokens")?;
        previous = word;
    }
    if total != snapshot.tokens {
        return Err("invalid_guest_index_tokens");
    }
    Ok(())
}

/// Evidence-only history. Never supplies a replacement artifact or chooses work.
#[derive(Default)]
pub struct History(BTreeMap<&'static str, String>);
impl History {
    pub fn observe(&mut self, payload: &str, snapshot: &Snapshot) -> Result<(), &'static str> {
        validate(payload, snapshot)?;
        let key = snapshot.record.ok_or("missing_guest_goal")?;
        match snapshot.event.as_str() {
            "executed" if !self.0.contains_key(key) => {
                self.0.insert(key, payload.to_owned());
            }
            "reused" if self.0.get(key).is_some_and(|old| old == payload) => {}
            _ => return Err("unproven_guest_index_reuse"),
        }
        if self.0.len() as u64 != snapshot.index.as_ref().unwrap().cached_records {
            return Err("unexpected_guest_cache");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot() -> Snapshot {
        Snapshot::parse("coordinator=EL0; worker=EL0; state=completed; event=executed; ticket=1; record=notes@1; bytes=30; tokens=7; completed=1; a_calls=10; b_calls=8; a_slices=0; b_slices=0; trace=AO,AH; operation=record.index; cached_records=1; cache_hits=0; output_bytes=33").unwrap()
    }
    const GOOD: &str = "record=notes@1; bytes=30; worker=EL0; state=completed; index=10:1,7:1,a:1,at:1,pump:2,review:1";
    #[test]
    fn artifact_must_match_the_observed_revision_and_canonical_shape() {
        let state = snapshot();
        assert!(validate(GOOD, &state).is_ok());
        for bad in [
            GOOD.replace("notes@1", "notes@2"),
            GOOD.replace("pump:2", "pump:1"),
            GOOD.replace("pump:2", "Pump:2"),
            GOOD.replace("10:1,7:1", "7:1,10:1"),
            GOOD.replace("completed", "failed"),
            GOOD.replace("bytes=30", "bytes=030"),
            format!("{GOOD}; bytes=30"),
            format!("{GOOD},x:1"),
        ] {
            assert!(validate(&bad, &state).is_err(), "{bad}");
        }
        // Shape checks are not semantic verification: the independent source checker
        // must reject a well-formed wrong token even when lengths/counts agree.
        assert!(validate(&GOOD.replace("pump", "wrong")[..], &state).is_err());
        assert!(validate(&GOOD.replace("pump", "jump"), &state).is_ok());
    }
    #[test]
    fn reuse_requires_an_identical_previously_observed_artifact() {
        let mut history = History::default();
        let mut state = snapshot();
        state.event = "reused".into();
        assert!(history.observe(GOOD, &state).is_err());
        state.event = "executed".into();
        history.observe(GOOD, &state).unwrap();
        assert!(history.observe(GOOD, &state).is_err());
        state.event = "reused".into();
        state.ticket = 3;
        state.index.as_mut().unwrap().cache_hits = 1;
        history.observe(GOOD, &state).unwrap();
        assert!(
            history
                .observe(&GOOD.replace("pump", "jump"), &state)
                .is_err()
        );
    }
}

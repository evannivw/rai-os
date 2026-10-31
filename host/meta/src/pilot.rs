//! Deterministic comparison rules. No clock, provider, UART or process dependency.
use crate::domain::{self, DecisionContext, JobRule, PilotJob, PilotView};
use crate::set_report;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const MAX_DECISIONS: usize = 8;
pub const GOAL: &str = "Complete the current required index set within the deadline and simulated credit cap. Maximize weighted on-time records only among valid complete sets; then minimize completion time and spent credits. Select one eligible missing record.";
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub at_ms: u64,
    pub credit_cap: u64,
    pub jobs: Vec<JobRule>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub id: String,
    pub deadline_ms: u64,
    pub events: Vec<Event>,
}
impl Scenario {
    pub fn parse(text: &str) -> Result<Self, &'static str> {
        let s: Self = serde_json::from_str(text).map_err(|_| "invalid_pilot_scenario")?;
        if s.id.is_empty()
            || s.id.len() > 64
            || !s.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !(200..=30_000).contains(&s.deadline_ms)
            || s.events.is_empty()
            || s.events.len() > 8
            || s.events[0].at_ms != 0
        {
            return Err("invalid_pilot_limits");
        }
        let mut previous = 0;
        for (i, e) in s.events.iter().enumerate() {
            if e.at_ms >= s.deadline_ms || (i > 0 && e.at_ms <= previous) || e.credit_cap > 32 {
                return Err("invalid_pilot_event");
            }
            set_report::keys(e.jobs.iter().map(|j| j.record.as_str()))?;
            if e.jobs.iter().any(|j| {
                j.due_ms == 0 || j.due_ms > s.deadline_ms || j.weight == 0 || j.weight > 100
            }) {
                return Err("invalid_job_rule");
            }
            previous = e.at_ms;
        }
        Ok(s)
    }
}
pub fn credits(key: &str) -> Result<u64, &'static str> {
    let bytes = domain::RECORDS
        .iter()
        .find(|(k, _)| *k == key)
        .ok_or("record_not_allowed")?
        .1;
    Ok(bytes.div_ceil(32).max(1))
}
pub fn context(
    set: &domain::IndexSet,
    e: &Event,
    ready: &BTreeMap<String, u64>,
    spent: u64,
    now: u64,
    deadline: u64,
) -> Result<DecisionContext, &'static str> {
    let mut c = set.context()?;
    c.authorized_records
        .retain(|(key, _)| credits(key).is_ok_and(|cost| spent + cost <= e.credit_cap));
    let mut jobs: Vec<_> = e
        .jobs
        .iter()
        .map(|j| {
            Ok(PilotJob {
                record: j.record.clone(),
                due_ms: j.due_ms,
                weight: j.weight,
                credits: credits(&j.record)?,
                ready_at_ms: ready.get(&j.record).copied(),
            })
        })
        .collect::<Result<_, &'static str>>()?;
    jobs.sort_by(|a, b| a.record.cmp(&b.record));
    c.pilot = Some(PilotView {
        elapsed_ms: now,
        deadline_ms: deadline,
        credit_cap: e.credit_cap,
        spent_credits: spent,
        jobs,
    });
    Ok(c)
}
/// Both rules re-evaluate current inputs after every event/result, share cache and
/// affordability filtering, and prioritize unexpired targets before late ones.
pub fn choose<'a>(policy: &str, c: &'a DecisionContext) -> Result<&'a str, &'static str> {
    let view = c.pilot.as_ref().ok_or("missing_pilot_context")?;
    if !matches!(policy, "edf" | "value") {
        return Err("unknown_fixed_policy");
    }
    let eligible = |j: &&PilotJob| c.authorized_records.iter().any(|(k, _)| *k == j.record);
    view.jobs
        .iter()
        .filter(eligible)
        .min_by(|a, b| {
            let lateness = (a.due_ms < view.elapsed_ms).cmp(&(b.due_ms < view.elapsed_ms));
            let ordering = if policy == "edf" {
                a.due_ms
                    .cmp(&b.due_ms)
                    .then_with(|| b.weight.cmp(&a.weight))
                    .then_with(|| a.credits.cmp(&b.credits))
            } else {
                // Exact rational comparison, no floating-point tie/rounding effects.
                (u64::from(b.weight) * a.credits)
                    .cmp(&(u64::from(a.weight) * b.credits))
                    .then_with(|| a.due_ms.cmp(&b.due_ms))
            };
            lateness
                .then(ordering)
                .then_with(|| a.record.cmp(&b.record))
        })
        .map(|j| j.record.as_str())
        .ok_or("no_affordable_work")
}
pub fn score(event: &Event, ready: &BTreeMap<String, u64>, passed: bool) -> u32 {
    if !passed {
        return 0;
    }
    event
        .jobs
        .iter()
        .filter(|j| ready.get(&j.record).is_some_and(|at| *at <= j.due_ms))
        .map(|j| j.weight)
        .sum()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn baselines_share_affordability_and_have_distinct_declared_ties() {
        let e = Event {
            at_ms: 0,
            credit_cap: 3,
            jobs: vec![
                JobRule {
                    record: "brief@1".into(),
                    due_ms: 1000,
                    weight: 10,
                },
                JobRule {
                    record: "empty@1".into(),
                    due_ms: 2000,
                    weight: 20,
                },
            ],
        };
        let set = domain::IndexSet {
            goal_version: 1,
            required: vec!["brief@1", "empty@1"],
            missing: vec!["brief@1", "empty@1"],
            completed: 0,
        };
        let c = context(&set, &e, &BTreeMap::new(), 0, 0, 3000).unwrap();
        assert_eq!(choose("edf", &c).unwrap(), "brief@1");
        assert_eq!(choose("value", &c).unwrap(), "empty@1");
        let c = context(&set, &e, &BTreeMap::new(), 2, 0, 3000).unwrap();
        assert_eq!(choose("edf", &c).unwrap(), "empty@1");
        assert!(
            context(&set, &e, &BTreeMap::new(), 3, 0, 3000)
                .unwrap()
                .authorized_records
                .is_empty()
        );
        assert_eq!(credits("empty@1"), Ok(1));
        assert_eq!(credits("notes@2"), Ok(2));
    }
    #[test]
    fn scenario_rejects_ambiguous_unbounded_and_unauthorized_rules() {
        let good = r#"{"id":"test","deadline_ms":4000,"events":[{"at_ms":0,"credit_cap":3,"jobs":[{"record":"notes@1","due_ms":2000,"weight":4}]}]}"#;
        assert!(Scenario::parse(good).is_ok());
        for bad in [
            good.replace("4000", "30001"),
            good.replace("notes@1", "restricted@1"),
            good.replace("\"weight\":4", "\"weight\":0"),
            good.replace("\"at_ms\":0", "\"at_ms\":1"),
            good.replace("\"credit_cap\":3", "\"credit_cap\":33"),
            good.replace("\"due_ms\":2000", "\"due_ms\":5000"),
        ] {
            assert!(Scenario::parse(&bad).is_err());
        }
    }
    #[test]
    fn final_score_requires_complete_success_and_exact_revision_deadline() {
        let e = Event {
            at_ms: 0,
            credit_cap: 2,
            jobs: vec![JobRule {
                record: "notes@2".into(),
                due_ms: 1000,
                weight: 9,
            }],
        };
        let mut ready = BTreeMap::from([("notes@1".into(), 1)]);
        assert_eq!(score(&e, &ready, true), 0);
        ready.insert("notes@2".into(), 1000);
        assert_eq!(score(&e, &ready, true), 9);
        assert_eq!(score(&e, &ready, false), 0);
        ready.insert("notes@2".into(), 1001);
        assert_eq!(score(&e, &ready, true), 0);
    }
}

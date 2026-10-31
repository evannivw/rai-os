//! Whole-set model choices; the host transports events and never supplies cached output.
use crate::{
    config,
    decision_source::{Pending, Source},
    domain::{self, Action, Work},
    guest_bridge::Transport,
    guest_report,
    model::Operation,
    provider, qemu,
    set_report::{self, Snapshot},
};
use serde::Deserialize;
use serde_json::json;
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};
const MAX_CALLS: usize = 8;
const GOAL: &str = "Complete the current RAI-OS required index set. Choose one missing revision for record.index with the supplied set state_version, or decline if unsupported. Do not recreate completed revisions.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Event {
    at_ms: u64,
    required: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    events: Vec<Event>,
}
impl Scenario {
    fn parse(text: &str) -> Result<Self, &'static str> {
        let s: Self = serde_json::from_str(text).map_err(|_| "invalid_set_scenario")?;
        if s.events.is_empty() || s.events.len() > 8 {
            return Err("scenario_requires_1_to_8_events");
        }
        let mut previous = 0;
        for e in &s.events {
            if e.at_ms < previous || e.at_ms > 30_000 {
                return Err("invalid_event_time");
            }
            set_report::keys(e.required.iter().map(String::as_str))?;
            previous = e.at_ms;
        }
        Ok(s)
    }
}
fn status(t: &mut Transport) -> Result<Snapshot, String> {
    let reply = t.exchange(&Operation::CoordBundleStatus, "coord.bundle.status", None)?;
    if !reply.success {
        return Err(reply.payload);
    }
    Ok(Snapshot::parse(&reply.payload)?)
}
fn apply_due(
    t: &mut Transport,
    scenario: &Scenario,
    cursor: &mut usize,
    clock: Instant,
    current: &mut Option<Snapshot>,
) -> Result<(), String> {
    while let Some(e) = scenario.events.get(*cursor) {
        if u128::from(e.at_ms) > clock.elapsed().as_millis() {
            break;
        }
        let keys = set_report::keys(e.required.iter().map(String::as_str))?;
        let argument = keys.join(",");
        println!(
            "{}",
            json!({"event":"set_due","index":*cursor,"required":keys,"scheduled_ms":e.at_ms,"schedule_elapsed_ms":clock.elapsed().as_millis()})
        );
        let reply = t.exchange(
            &Operation::CoordGoal(&argument),
            "coord.goal",
            Some(&argument),
        )?;
        if !reply.success {
            return Err(reply.payload);
        }
        let next = Snapshot::parse(&reply.payload)?;
        if next.required != keys
            || current.as_ref().is_some_and(|old| {
                next.goal_version <= old.goal_version || next.completed != old.completed
            })
        {
            return Err("unexpected_goal_transition".into());
        }
        *current = Some(next);
        *cursor += 1;
    }
    Ok(())
}
fn session(t: &mut Transport, scenario: &Scenario, source: &mut Source) -> Result<(), String> {
    let start = t.snapshot(&Operation::CoordIndexStart, "coord.start", Some("index"))?;
    if start.state != "idle" || start.completed != 0 {
        return Err("guest_not_fresh".into());
    }
    let clock = Instant::now();
    let (mut cursor, mut calls) = (0, 0);
    let mut current = None;
    let mut pending: Option<(Pending, Snapshot)> = None;
    let mut published = None;
    loop {
        apply_due(t, scenario, &mut cursor, clock, &mut current)?;
        let answer = if let Some((waiting, _)) = &pending {
            waiting.poll()?
        } else {
            None
        };
        if let Some(answer) = answer {
            let (waiting, requested) = pending.take().unwrap();
            println!(
                "{}",
                json!({"event":"decision_response","source":source.name(),"call_id":calls,"snapshot_goal":waiting.version,"latency_ms":answer.latency_ms,"usage":answer.usage})
            );
            let action = domain::validate_for(&answer.proposal, waiting.version, Work::Index)?;
            drop(waiting);
            apply_due(t, scenario, &mut cursor, clock, &mut current)?;
            let old = current.as_ref().ok_or("missing_current_set")?.clone();
            match action {
                Action::Decline => {
                    let obsolete = old.goal_version != requested.goal_version;
                    println!(
                        "{}",
                        json!({"event":"proposal_declined","call_id":calls,"proposal":answer.proposal,"obsolete":obsolete})
                    );
                    if !obsolete {
                        return Err("model_declined_current_set".into());
                    }
                }
                Action::Index { key, .. } => {
                    // Bind to the ORIGINAL missing choices, never a newer goal's permissions.
                    if !requested.missing.contains(&key) {
                        return Err("proposal_not_in_requested_missing_set".into());
                    }
                    let argument = format!("{}:{key}", requested.goal_version);
                    println!(
                        "{}",
                        json!({"event":"proposal_forwarded","call_id":calls,"proposal":answer.proposal})
                    );
                    let reply = t.exchange(
                        &Operation::CoordSelect(requested.goal_version, key),
                        "coord.select",
                        Some(&argument),
                    )?;
                    if requested.goal_version != old.goal_version {
                        if reply.success || reply.payload != "coord_stale_goal" || status(t)? != old
                        {
                            return Err("unexpected_stale_goal_result".into());
                        }
                    } else {
                        if !reply.success {
                            return Err(reply.payload);
                        }
                        let selected = guest_report::Snapshot::parse(&reply.payload)?;
                        if selected.state != "awaiting_proposal"
                            || selected.event != "observed"
                            || selected.record != Some(key)
                            || selected.completed != old.completed
                            || selected.ticket <= old.goal_version
                            || selected.a_calls != 2
                            || selected.b_calls != 0
                        {
                            return Err("unexpected_selection_result".into());
                        }
                        // No event ingress between select/propose. A goal change from another
                        // authorized transport would still invalidate this per-record ticket.
                        let argument = format!("{}:{key}", selected.ticket);
                        let next = t.snapshot(
                            &Operation::CoordPropose(selected.ticket, key),
                            "coord.propose",
                            Some(&argument),
                        )?;
                        next.check_completion(&selected, &action, selected.ticket)?;
                        t.artifact(&next)?;
                        let after = status(t)?;
                        let expected: Vec<_> =
                            old.missing.iter().copied().filter(|k| *k != key).collect();
                        if after.goal_version != old.goal_version
                            || after.required != old.required
                            || after.missing != expected
                            || after.completed != old.completed + 1
                        {
                            return Err("unexpected_set_progress".into());
                        }
                        current = Some(after);
                    }
                }
                Action::Count { .. } => return Err("operation_not_allowed".into()),
            }
        }
        // Apply arrivals after synchronous guest work before publishing/requesting more work.
        apply_due(t, scenario, &mut cursor, clock, &mut current)?;
        if let Some(now) = &current {
            if now.missing.is_empty() && published != Some(now.goal_version) {
                let reply = t.exchange(&Operation::CoordBundle, "coord.bundle", None)?;
                if !reply.success {
                    return Err(reply.payload);
                }
                now.check_artifact(&reply.payload)?;
                println!(
                    "{}",
                    json!({"event":"guest_bundle","snapshot":now,"payload":reply.payload})
                );
                published = Some(now.goal_version);
            }
            if pending.is_none() && !now.missing.is_empty() {
                if calls == MAX_CALLS {
                    return Err("model_call_budget_exhausted".into());
                }
                let context = now.context()?;
                calls += 1;
                println!(
                    "{}",
                    json!({"event":"decision_requested","call_id":calls,"snapshot":context,"elapsed_ms":t.started.elapsed().as_millis()})
                );
                pending = Some((source.start(GOAL, context)?, now.clone()));
            }
            if cursor == scenario.events.len() && pending.is_none() && now.missing.is_empty() {
                if source.has_unused() {
                    return Err("unused_replay_proposals".into());
                }
                println!(
                    "{}",
                    json!({"event":"set_bridge_finished","status":"observed_current_set","model_calls":calls,"snapshot":now,"elapsed_ms":t.started.elapsed().as_millis()})
                );
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}
pub fn run(kernel: &str, path: &Path, replay: Option<&Path>, graphics: bool) -> Result<(), String> {
    let scenario = Scenario::parse(&config::bounded_file(path, 8192)?)?;
    let mut source = Source::load(replay, MAX_CALLS)?;
    println!(
        "{}",
        json!({"event":"set_bridge_started","source":source.name(),"model":if source.is_live(){Some(provider::MODEL)}else{None},"graphics":graphics,"event_source":"scripted_required_sets","goal":GOAL,"max_model_calls":MAX_CALLS})
    );
    let started = Instant::now();
    let connection = if graphics {
        qemu::Connection::start_graphics(kernel)?
    } else {
        qemu::Connection::start(kernel)?
    };
    println!("{}", json!({"event":"guest_boot_confirmed"}));
    let mut t = Transport {
        connection,
        request_id: 0,
        started,
        work: Work::Index,
        artifacts: crate::guest_index::History::default(),
    };
    let result = session(&mut t, &scenario, &mut source);
    let cleanup = t
        .snapshot(&Operation::CoordStop, "coord.stop", None)
        .and_then(|s| {
            if s.state == "stopped" {
                Ok(())
            } else {
                Err("guest_stop_failed".into())
            }
        });
    if let Err(error) = cleanup {
        println!("{}", json!({"event":"guest_cleanup_failed","error":error}));
        return result.and(Err("guest_cleanup_failed".into()));
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_events_cannot_change_capabilities() {
        for text in [
            r#"{"events":[]}"#,
            r#"{"events":[{"at_ms":0,"required":["restricted@1"]}]}"#,
            r#"{"events":[{"at_ms":30001,"required":["notes@1"]}]}"#,
            r#"{"events":[{"at_ms":0,"required":["notes@1","notes@2"]}]}"#,
        ] {
            assert!(Scenario::parse(text).is_err());
        }
        assert!(
            Scenario::parse(r#"{"events":[{"at_ms":0,"required":["brief@1","notes@1"]}]}"#).is_ok()
        );
    }
}

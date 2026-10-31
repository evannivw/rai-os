//! Host transport for the guest coordinator. Goal/admission state remains at EL0.
use crate::{
    config,
    decision_source::{Pending, Source},
    domain::{self, Action, Work},
    guest_report::Snapshot,
    model::{Operation, Reply},
    provider, qemu,
};
use serde::Deserialize;
use serde_json::json;
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};
const MAX_CALLS: usize = 4;
const INDEX_GOAL: &str = "Ask the RAI-OS worker to build an exact token-frequency index for required_record. Propose record.index with the supplied state_version, or decline if unsupported.";
const GOAL: &str = "Complete the current immutable record task requested by RAI-OS. Propose record.count for required_record with the supplied state_version, or decline if unsupported.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Event {
    at_ms: u64,
    record: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    events: Vec<Event>,
}
impl Scenario {
    fn parse(text: &str) -> Result<Self, &'static str> {
        let scenario: Self = serde_json::from_str(text).map_err(|_| "invalid_guest_scenario")?;
        if scenario.events.is_empty() || scenario.events.len() > 8 {
            return Err("scenario_requires_1_to_8_events");
        }
        let mut previous = 0;
        for event in &scenario.events {
            if event.at_ms < previous || event.at_ms > 30_000 {
                return Err("invalid_event_time");
            }
            if !domain::RECORDS.iter().any(|(key, _)| *key == event.record) {
                return Err("record_not_allowed");
            }
            previous = event.at_ms;
        }
        Ok(scenario)
    }
}
struct Transport {
    connection: qemu::Connection,
    request_id: u32,
    started: Instant,
    work: Work,
    artifacts: crate::guest_index::History,
}
impl Transport {
    fn exchange(
        &mut self,
        operation: &Operation<'_>,
        name: &str,
        argument: Option<&str>,
    ) -> Result<Reply, String> {
        self.request_id = self
            .request_id
            .checked_add(1)
            .ok_or("request_id_exhausted")?;
        let reply = self.connection.request(self.request_id, operation)?;
        println!(
            "{}",
            json!({"event":"guest_exchange","request_id":self.request_id,"operation":name,
            "argument":argument,"success":reply.success,"payload":reply.payload,"elapsed_ms":self.started.elapsed().as_millis()})
        );
        Ok(reply)
    }
    fn artifact(&mut self, snapshot: &Snapshot) -> Result<(), String> {
        if self.work != Work::Index {
            return Ok(());
        }
        let reply = self.exchange(&Operation::CoordResult, "coord.result", None)?;
        if !reply.success {
            return Err("guest_artifact_unavailable".into());
        }
        self.artifacts.observe(&reply.payload, snapshot)?;
        println!(
            "{}",
            json!({"event":"guest_index","ticket":snapshot.ticket,
            "record":snapshot.record,"origin":snapshot.event,"payload":reply.payload})
        );
        Ok(())
    }
    fn snapshot(
        &mut self,
        operation: &Operation<'_>,
        name: &str,
        argument: Option<&str>,
    ) -> Result<Snapshot, String> {
        let reply = self.exchange(operation, name, argument)?;
        if !reply.success {
            return Err("guest_rejected_control_operation".into());
        }
        let snapshot = Snapshot::parse(&reply.payload)?;
        if name != "coord.stop" && snapshot.work() != self.work {
            return Err("unexpected_guest_operation".into());
        }
        Ok(snapshot)
    }
}
fn apply_due(
    transport: &mut Transport,
    scenario: &Scenario,
    cursor: &mut usize,
    event_clock: Instant,
    current: &mut Snapshot,
) -> Result<(), String> {
    while let Some(event) = scenario.events.get(*cursor) {
        let elapsed = event_clock.elapsed().as_millis();
        if u128::from(event.at_ms) > elapsed {
            break;
        }
        println!(
            "{}",
            json!({"event":"observation_due","index":*cursor,"record":event.record,
            "scheduled_ms":event.at_ms,"schedule_elapsed_ms":elapsed})
        );
        let next = transport.snapshot(
            &Operation::CoordObserve(&event.record),
            "coord.observe",
            Some(&event.record),
        )?;
        next.check_observation(current, &event.record)?;
        if next.event == "reused" {
            transport.artifact(&next)?;
        }
        *current = next;
        *cursor += 1;
    }
    Ok(())
}
struct Outcome {
    status: &'static str,
    calls: usize,
    snapshot: Snapshot,
    remaining_events: usize,
}
fn session(
    transport: &mut Transport,
    scenario: &Scenario,
    source: &mut Source,
) -> Result<Outcome, String> {
    let index = transport.work == Work::Index;
    let mut current = transport.snapshot(
        &if index {
            Operation::CoordIndexStart
        } else {
            Operation::CoordStart
        },
        "coord.start",
        if index { Some("index") } else { None },
    )?;
    if current.state != "idle" {
        return Err("guest_not_idle_after_start".into());
    }
    let event_clock = Instant::now();
    let mut cursor = 0;
    let mut calls = 0;
    let mut pending: Option<Pending> = None;
    loop {
        apply_due(transport, scenario, &mut cursor, event_clock, &mut current)?;
        let response = if let Some(waiting) = &pending {
            waiting.poll()?.map(|answer| (waiting.version, answer))
        } else {
            None
        };
        if let Some((ticket, answer)) = response {
            drop(pending.take());
            println!(
                "{}",
                json!({"event":"decision_response","source":source.name(),"call_id":calls,
                "snapshot_ticket":ticket,"latency_ms":answer.latency_ms,"usage":answer.usage})
            );
            // Validate shape/capabilities AND correlation to the original request.
            // Current-goal freshness is still decided by the guest, not by a host cache.
            let action = domain::validate_for(&answer.proposal, ticket, transport.work)?;
            let ticket = u32::try_from(ticket)
                .ok()
                .filter(|t| *t != 0)
                .ok_or("invalid_proposal_ticket")?;
            apply_due(transport, scenario, &mut cursor, event_clock, &mut current)?;
            if matches!(action, Action::Decline) {
                println!(
                    "{}",
                    json!({"event":"proposal_declined","call_id":calls,"proposal":answer.proposal,
                    "obsolete":current.ticket != ticket})
                );
                if current.ticket == ticket {
                    return Ok(Outcome {
                        status: "declined",
                        calls,
                        snapshot: current,
                        remaining_events: scenario.events.len() - cursor,
                    });
                }
                continue;
            }
            let key = match &action {
                Action::Count { key, .. } | Action::Index { key, .. } => key,
                Action::Decline => unreachable!(),
            };
            let argument = format!("{ticket}:{key}");
            println!(
                "{}",
                json!({"event":"proposal_forwarded","call_id":calls,"proposal":answer.proposal})
            );
            let reply = transport.exchange(
                &Operation::CoordPropose(ticket, key),
                "coord.propose",
                Some(&argument),
            )?;
            if reply.success {
                let next = Snapshot::parse(&reply.payload)?;
                next.check_completion(&current, &action, ticket)?;
                transport.artifact(&next)?;
                current = next;
            } else {
                let next = transport.snapshot(&Operation::CoordStatus, "coord.status", None)?;
                let expected = if ticket != current.ticket {
                    "coord_stale_proposal"
                } else if current.record != Some(*key) {
                    "coord_wrong_record"
                } else if current.state == "completed" {
                    "coord_already_completed"
                } else {
                    return Err("guest_rejected_matching_proposal".into());
                };
                if reply.payload != expected
                    || next.event != expected.trim_start_matches("coord_")
                    || !next.same_goal_result(&current)
                    || next.b_calls != 0
                    || next.b_slices != 0
                    || next.a_calls != 2
                {
                    return Err("unexpected_guest_rejection".into());
                }
                current = next;
                // A stale result is normal during new events. Other bad proposals stop;
                // there are no silent model retries or deterministic substitute results.
                if reply.payload != "coord_stale_proposal" {
                    return Err(reply.payload);
                }
            }
        }
        // Read reports emitted by the guest; never predict its goal version locally.
        if pending.is_none() && current.state == "awaiting_proposal" {
            if calls == MAX_CALLS {
                return Err("model_call_budget_exhausted".into());
            }
            let context = current.context()?;
            calls += 1;
            println!(
                "{}",
                json!({"event":"decision_requested","call_id":calls,"snapshot":context,
                "elapsed_ms":transport.started.elapsed().as_millis()})
            );
            pending = Some(source.start(if index { INDEX_GOAL } else { GOAL }, context)?);
        }
        if cursor == scenario.events.len() && pending.is_none() && current.state == "completed" {
            if source.has_unused() {
                return Err("unused_replay_proposals".into());
            }
            return Ok(Outcome {
                status: "observed_current_goal",
                calls,
                snapshot: current,
                remaining_events: 0,
            });
        }
        thread::sleep(Duration::from_millis(10));
    }
}
pub fn run(
    kernel: &str,
    path: &Path,
    replay: Option<&Path>,
    work: Work,
    graphics: bool,
) -> Result<(), String> {
    let scenario = Scenario::parse(&config::bounded_file(path, 8192)?)?;
    let mut source = Source::load(replay, MAX_CALLS)?;
    println!(
        "{}",
        json!({"event":"guest_bridge_started","source":source.name(),"model":if source.is_live(){Some(provider::MODEL)}else{None},
        "operation":work.name(),"graphics":graphics,"event_source":"scripted_observations","goal":if work == Work::Index { INDEX_GOAL } else { GOAL },"max_model_calls":MAX_CALLS,"state_owner":"guest_EL0"})
    );
    let started = Instant::now();
    let connection = if graphics {
        qemu::Connection::start_graphics(kernel)?
    } else {
        qemu::Connection::start(kernel)?
    };
    println!(
        "{}",
        json!({"event":"guest_boot_confirmed","elapsed_ms":started.elapsed().as_millis()})
    );
    let mut transport = Transport {
        connection,
        request_id: 0,
        started,
        work,
        artifacts: crate::guest_index::History::default(),
    };
    let result = session(&mut transport, &scenario, &mut source);
    let cleanup = transport
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
    let outcome = result?;
    println!(
        "{}",
        json!({"event":"guest_bridge_finished","status":outcome.status,"model_calls":outcome.calls,
        "snapshot":outcome.snapshot,"remaining_events":outcome.remaining_events,"elapsed_ms":started.elapsed().as_millis()})
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observations_are_bounded_and_reject_unauthorized_records_before_boot() {
        for text in [
            r#"{"events":[]}"#,
            r#"{"events":[{"at_ms":30001,"record":"notes@1"}]}"#,
            r#"{"events":[{"at_ms":0,"record":"restricted@1"}]}"#,
            r#"{"events":[{"at_ms":2,"record":"notes@1"},{"at_ms":1,"record":"notes@2"}]}"#,
            r#"{"events":[{"at_ms":0,"record":"notes@1","shell":"bad"}]}"#,
        ] {
            assert!(Scenario::parse(text).is_err());
        }
        assert!(
            Scenario::parse(
                r#"{"events":[{"at_ms":0,"record":"brief@1"},{"at_ms":0,"record":"empty@1"}]}"#
            )
            .is_ok()
        );
    }
}

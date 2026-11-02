//! One model plan per snapshot; deterministic, individually admitted guest jobs.
use crate::{
    batch::{self, Plan},
    batch_publication, config,
    decision_source::{Pending, Source},
    domain::{Action, Work},
    guest_bridge::Transport,
    model::Operation,
    provider, qemu,
    set_bridge::{Scenario, apply_due, status},
    set_report::Snapshot,
};
use serde_json::json;
use std::{
    collections::VecDeque,
    path::Path,
    thread,
    time::{Duration, Instant},
};
const MAX_CALLS: usize = 8;
const GOAL: &str = "Complete the current required index set with one plan containing every missing record.index operation. Preserve the supplied state_version; decline if unsupported.";
struct Queue {
    version: u32,
    keys: VecDeque<&'static str>,
}
fn execute(t: &mut Transport, old: &Snapshot, key: &'static str) -> Result<Snapshot, String> {
    // Every action is rechecked against observed current missing work and admitted
    // by the guest's original goal generation; the plan itself is never executable code.
    let before = status(t)?;
    if before != *old || !before.missing.contains(&key) {
        return Err("batch_state_changed".into());
    }
    let arg = format!("{}:{key}", old.goal_version);
    let selected = t.snapshot(
        &Operation::CoordSelect(old.goal_version, key),
        "coord.select",
        Some(&arg),
    )?;
    if selected.state != "awaiting_proposal"
        || selected.event != "observed"
        || selected.record != Some(key)
        || selected.completed != old.completed
        || selected.ticket <= old.goal_version
        || selected.a_calls != 2
        || selected.b_calls != 0
    {
        return Err("unexpected_batch_selection".into());
    }
    // This single transport does not ingest events between select/propose. The
    // guest still checks the per-record ticket independently of host validation.
    let arg = format!("{}:{key}", selected.ticket);
    let next = t.snapshot(
        &Operation::CoordPropose(selected.ticket, key),
        "coord.propose",
        Some(&arg),
    )?;
    let bytes = crate::domain::RECORDS
        .iter()
        .find(|(k, _)| *k == key)
        .ok_or("record_not_allowed")?
        .1;
    next.check_completion(&selected, &Action::Index { key, bytes }, selected.ticket)?;
    t.artifact(&next)?;
    let after = status(t)?;
    let missing: Vec<_> = old.missing.iter().copied().filter(|k| *k != key).collect();
    if after.goal_version != old.goal_version
        || after.required != old.required
        || after.missing != missing
        || after.completed != old.completed + 1
    {
        return Err("unexpected_batch_progress".into());
    }
    Ok(after)
}
fn session(
    t: &mut Transport,
    scenario: &Scenario,
    source: &mut Source<Plan>,
) -> Result<(), String> {
    let start = t.snapshot(&Operation::CoordIndexStart, "coord.start", Some("index"))?;
    if start.state != "idle" || start.completed != 0 {
        return Err("guest_not_fresh".into());
    }
    let clock = Instant::now();
    let (mut cursor, mut calls) = (0, 0);
    let mut current = None;
    let mut pending: Option<(Pending<Plan>, Snapshot)> = None;
    let mut queue: Option<Queue> = None;
    let mut published = None;
    loop {
        apply_due(t, scenario, &mut cursor, clock, &mut current)?;
        if let Some(q) = &queue
            && current.as_ref().ok_or("missing_current_set")?.goal_version != q.version
        {
            println!(
                "{}",
                json!({"event":"batch_discarded","stage":"execution","state_version":q.version,"remaining":q.keys.len()})
            );
            queue = None;
        }
        if let Some(answer) = pending
            .as_ref()
            .map(|(p, _)| p.poll())
            .transpose()?
            .flatten()
        {
            let (waiting, requested) = pending.take().ok_or("missing_pending_batch")?;
            println!(
                "{}",
                json!({"event":"batch_response","source":source.name(),"call_id":calls,"snapshot_goal":waiting.version,"latency_ms":answer.latency_ms,"usage":answer.usage})
            );
            // Full validation precedes ALL dispatch, including invalid suffixes.
            let keys = batch::validate(&answer.proposal, &requested)?;
            drop(waiting);
            apply_due(t, scenario, &mut cursor, clock, &mut current)?;
            let now = current.as_ref().ok_or("missing_current_set")?;
            if now.goal_version != requested.goal_version {
                println!(
                    "{}",
                    json!({"event":"batch_discarded","stage":"response","state_version":requested.goal_version,"remaining":keys.as_ref().map_or(0,Vec::len)})
                );
            } else if let Some(keys) = keys {
                println!(
                    "{}",
                    json!({"event":"batch_validated","call_id":calls,"plan":answer.proposal})
                );
                queue = Some(Queue {
                    version: requested.goal_version,
                    keys: keys.into(),
                });
            } else {
                return Err("model_declined_current_batch".into());
            }
        }
        // Reconcile even when no provider is pending. Only one job per turn means
        // events due during a job invalidate the remaining plan before its next job.
        apply_due(t, scenario, &mut cursor, clock, &mut current)?;
        if let Some(q) = &mut queue {
            let old = current.as_ref().ok_or("missing_current_set")?;
            if old.goal_version != q.version {
                continue;
            }
            if let Some(key) = q.keys.pop_front() {
                println!(
                    "{}",
                    json!({"event":"batch_step","state_version":q.version,"record":key,"remaining":q.keys.len()})
                );
                current = Some(execute(t, old, key)?);
            }
            if q.keys.is_empty() {
                queue = None;
            }
        }
        apply_due(t, scenario, &mut cursor, clock, &mut current)?;
        if let Some(now) = current.clone() {
            if now.missing.is_empty() && published != Some(now.goal_version) {
                let candidate = batch_publication::prepare(t, &now)?;
                // A change arriving while the independent checker runs invalidates
                // its acknowledgment. No old artifact can be relabeled current.
                apply_due(t, scenario, &mut cursor, clock, &mut current)?;
                if current.as_ref() != Some(&now) {
                    println!(
                        "{}",
                        json!({"event":"batch_discarded","stage":"publication","state_version":now.goal_version,"remaining":0})
                    );
                    continue;
                }
                batch_publication::commit(t, &now, candidate)?;
                published = Some(now.goal_version);
            }
            if pending.is_none() && queue.is_none() && !now.missing.is_empty() {
                if calls == MAX_CALLS {
                    return Err("model_call_budget_exhausted".into());
                }
                calls += 1;
                let context = now.context()?;
                println!(
                    "{}",
                    json!({"event":"batch_requested","call_id":calls,"snapshot":context,"elapsed_ms":t.started.elapsed().as_millis()})
                );
                pending = Some((source.start_batch(GOAL, context)?, now));
            }
        }
        if let Some(now) = &current
            && cursor == scenario.len()
            && pending.is_none()
            && queue.is_none()
            && published == Some(now.goal_version)
        {
            if source.has_unused() {
                return Err("unused_replay_proposals".into());
            }
            println!(
                "{}",
                json!({"event":"batch_finished","status":"verified_current_publication","model_calls":calls,"snapshot":now,"elapsed_ms":t.started.elapsed().as_millis()})
            );
            return Ok(());
        }
        // This is a demo event-loop polling interval, not a CPU scheduler quantum.
        thread::sleep(Duration::from_millis(10));
    }
}
pub fn run(kernel: &str, path: &Path, replay: Option<&Path>, graphics: bool) -> Result<(), String> {
    let scenario = Scenario::parse(&config::bounded_file(path, 8192)?)?;
    let mut source = Source::<Plan>::load(replay, MAX_CALLS)?;
    println!(
        "{}",
        json!({"event":"batch_started","source":source.name(),"model":if source.is_live(){Some(provider::MODEL)}else{None},"graphics":graphics,"event_source":"scripted_required_sets","goal":GOAL,"max_model_calls":MAX_CALLS,"max_operations":4})
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

/// Execute a host-approved semantic requirement using the same per-job guest
/// gates and independent publication adapter, with no second inference call.
/// This mode boots a fresh session; it does not implement semantic event recovery.
pub fn run_selected(kernel: &str, keys: &[&'static str], graphics: bool) -> Result<(), String> {
    let required = crate::set_report::keys(keys.iter().copied())?;
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
    let result = (|| -> Result<(), String> {
        let start = t.snapshot(&Operation::CoordIndexStart, "coord.start", Some("index"))?;
        if start.state != "idle" || start.completed != 0 {
            return Err("guest_not_fresh".into());
        }
        let argument = required.join(",");
        let reply = t.exchange(
            &Operation::CoordGoal(&argument),
            "coord.goal",
            Some(&argument),
        )?;
        if !reply.success {
            return Err(reply.payload);
        }
        let mut snapshot = Snapshot::parse(&reply.payload)?;
        if snapshot.required != required || snapshot.missing != required || snapshot.completed != 0
        {
            return Err("unexpected_semantic_goal".into());
        }
        for key in &required {
            snapshot = execute(&mut t, &snapshot, key)?;
        }
        let candidate = batch_publication::prepare(&mut t, &snapshot)?;
        if status(&mut t)? != snapshot {
            return Err("semantic_goal_changed".into());
        }
        batch_publication::commit(&mut t, &snapshot, candidate)?;
        println!(
            "{}",
            json!({"event":"semantic_finished","snapshot":snapshot,"elapsed_ms":started.elapsed().as_millis()})
        );
        Ok(())
    })();
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

/// A presentation replay, not a policy timing experiment. Previous publication
/// stays readable while the second approved goal reuses exact cached revisions.
pub fn run_sequence(
    kernel: &str,
    sets: &[Vec<&'static str>],
    graphics: bool,
) -> Result<(), String> {
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
    let pause = || {
        if graphics {
            thread::sleep(Duration::from_millis(1500));
        }
    };
    let result = (|| -> Result<(), String> {
        let start = t.snapshot(&Operation::CoordIndexStart, "coord.start", Some("index"))?;
        if start.state != "idle" || start.completed != 0 {
            return Err("guest_not_fresh".into());
        }
        let mut previous: Option<(u32, String)> = None;
        for (index, keys) in sets.iter().enumerate() {
            let argument = keys.join(",");
            let reply = t.exchange(
                &Operation::CoordGoal(&argument),
                "coord.goal",
                Some(&argument),
            )?;
            if !reply.success {
                return Err(reply.payload);
            }
            let mut snapshot = Snapshot::parse(&reply.payload)?;
            if snapshot.required != *keys {
                return Err("unexpected_sequence_goal".into());
            }
            println!(
                "{}",
                json!({"event":"sequence_goal","index":index,"snapshot":snapshot})
            );
            if let Some((old_version, old_bytes)) = &previous {
                let reply = t.exchange(&Operation::PublicationRead, "coord.publish.read", None)?;
                if !reply.success || reply.payload != *old_bytes {
                    return Err("historical_publication_changed".into());
                }
                let status =
                    t.exchange(&Operation::PublicationStatus, "coord.publish.status", None)?;
                let fields = batch_publication::fields(&status.payload)?;
                if !status.success
                    || fields["availability"] != "historical"
                    || fields["published_version"] != old_version.to_string()
                    || fields["target_version"] != snapshot.goal_version.to_string()
                {
                    return Err("unexpected_historical_status".into());
                }
                println!(
                    "{}",
                    json!({"event":"sequence_historical","published_version":old_version,"target_version":snapshot.goal_version,"unchanged":true})
                );
            }
            pause();
            for key in snapshot.missing.clone() {
                snapshot = execute(&mut t, &snapshot, key)?;
            }
            let candidate = batch_publication::prepare(&mut t, &snapshot)?;
            pause();
            previous = Some((snapshot.goal_version, candidate.payload.clone()));
            batch_publication::commit(&mut t, &snapshot, candidate)?;
            pause();
        }
        println!(
            "{}",
            json!({"event":"sequence_finished","snapshot":status(&mut t)?,"live_inference":false})
        );
        Ok(())
    })();
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

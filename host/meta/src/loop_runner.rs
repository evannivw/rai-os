//! Host event adapter and bounded coordination loop. One provider request at a time.
use crate::{
    config,
    coordinator::{self, Scenario, State},
    decision_source::{Pending, Source},
    domain::{self, Action},
    model, provider, qemu,
};
use serde_json::json;
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};
fn apply_due(
    state: &mut State,
    scenario: &Scenario,
    cursor: &mut usize,
    started: Instant,
) -> Result<(), String> {
    while let Some(event) = scenario.events.get(*cursor) {
        let elapsed = started.elapsed().as_millis();
        if u128::from(event.at_ms) > elapsed {
            break;
        }
        let changed = state.release(&event.record)?;
        println!(
            "{}",
            json!({"event":"release_applied","index":*cursor,"scheduled_ms":event.at_ms,"elapsed_ms":elapsed,"record":event.record,"changed":changed,"state_version":state.version()})
        );
        *cursor += 1;
    }
    Ok(())
}
pub fn run(kernel: &str, path: &Path, replay: Option<&Path>) -> Result<(), String> {
    let scenario = Scenario::parse(&config::bounded_file(path, 8192)?)?;
    let mut source = Source::load(replay, coordinator::MAX_CALLS)?;
    println!(
        "{}",
        json!({"event":"coordinator_started","source":source.name(),"model":if source.is_live(){Some(provider::MODEL)}else{None},"event_source":"scripted_release_fixture","goal":coordinator::GOAL,"max_model_calls":coordinator::MAX_CALLS})
    );
    let started = Instant::now();
    let mut state = State::default();
    let mut cursor = 0;
    let mut calls = 0;
    let mut pending: Option<Pending> = None;
    let mut connection: Option<qemu::Connection> = None;
    let mut request_id = 0_u32;
    loop {
        apply_due(&mut state, &scenario, &mut cursor, started)?;
        let response = if let Some(waiting) = &pending {
            waiting.poll()?.map(|answer| (waiting.version, answer))
        } else {
            None
        };
        if let Some((version, answer)) = response {
            drop(pending.take());
            println!(
                "{}",
                json!({"event":"decision_response","source":source.name(),"call_id":calls,"snapshot_version":version,"latency_ms":answer.latency_ms,"usage":answer.usage})
            );
            apply_due(&mut state, &scenario, &mut cursor, started)?;
            if version != state.version() {
                println!(
                    "{}",
                    json!({"event":"stale_proposal_discarded","call_id":calls,"snapshot_version":version,"state_version":state.version()})
                );
                continue;
            }
            let action = state.admit(&answer.proposal)?;
            if matches!(action, Action::Decline) {
                println!(
                    "{}",
                    json!({"event":"coordinator_finished","status":"declined","model_calls":calls,"snapshot":state.snapshot(),"remaining_events":scenario.events.len()-cursor})
                );
                return Ok(());
            }
            if connection.is_none() {
                connection = Some(qemu::Connection::start(kernel)?);
                println!("{}", json!({"event":"guest_boot_confirmed"}));
            }
            // Boot can take time too: apply due events again before UART admission.
            apply_due(&mut state, &scenario, &mut cursor, started)?;
            if version != state.version() {
                println!(
                    "{}",
                    json!({"event":"stale_proposal_discarded","call_id":calls,"snapshot_version":version,"state_version":state.version()})
                );
                continue;
            }
            let Action::Count { key, .. } = action else {
                unreachable!()
            };
            request_id += 1;
            println!(
                "{}",
                json!({"event":"operation_dispatched","call_id":calls,"request_id":request_id,"proposal":answer.proposal,"elapsed_ms":started.elapsed().as_millis()})
            );
            let reply = connection
                .as_mut()
                .ok_or("missing_guest_connection")?
                .request(request_id, &model::Operation::RecordCount(key))?;
            if !reply.success {
                return Err("guest_rejected_operation".into());
            }
            let observed = domain::observe(&action, &reply.payload)?;
            apply_due(&mut state, &scenario, &mut cursor, started)?;
            state.complete(observed.clone())?;
            println!(
                "{}",
                json!({"event":"observation_recorded","request_id":request_id,"observed":observed,"payload":reply.payload,"state_version":state.version(),"pending_record":state.pending(),"elapsed_ms":started.elapsed().as_millis()})
            );
        }
        if pending.is_none() && state.pending().is_some() {
            if calls == coordinator::MAX_CALLS {
                return Err("model_call_budget_exhausted".into());
            }
            calls += 1;
            let snapshot = state.snapshot();
            println!(
                "{}",
                json!({"event":"decision_requested","call_id":calls,"snapshot":snapshot,"elapsed_ms":started.elapsed().as_millis()})
            );
            pending = Some(source.start(coordinator::GOAL, snapshot)?);
        }
        if cursor == scenario.events.len() && pending.is_none() && state.pending().is_none() {
            if source.has_unused() {
                return Err("unused_replay_proposals".into());
            }
            println!(
                "{}",
                json!({"event":"coordinator_finished","status":"observed_current_revision","model_calls":calls,"snapshot":state.snapshot(),"elapsed_ms":started.elapsed().as_millis()})
            );
            return Ok(());
        }
        // Small host wait, not a busy kernel loop. No model call while idle.
        thread::sleep(Duration::from_millis(10));
    }
}

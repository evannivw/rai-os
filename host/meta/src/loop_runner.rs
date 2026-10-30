//! Host event adapter and bounded coordination loop. One provider request at a time.
use crate::{
    config::{self, Secret},
    coordinator::{self, Scenario, State},
    domain::{self, Action, Proposal},
    model, provider, qemu,
};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::VecDeque,
    path::Path,
    sync::{
        Arc,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replay {
    delay_ms: u64,
    proposal: Proposal,
}
enum Source {
    Meta(Arc<Secret>),
    Replay(VecDeque<Replay>),
}
impl Source {
    fn name(&self) -> &'static str {
        match self {
            Self::Meta(_) => "meta",
            Self::Replay(_) => "replay",
        }
    }
    fn start(&mut self, context: domain::DecisionContext) -> Result<Pending, String> {
        let version = context.state_version;
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = match self {
            Self::Meta(key) => {
                let key = Arc::clone(key);
                thread::spawn(move || {
                    let _ =
                        sender.send(provider::propose_context(&key, coordinator::GOAL, &context));
                })
            }
            Self::Replay(entries) => {
                let entry = entries.pop_front().ok_or("replay_exhausted")?;
                thread::spawn(move || {
                    let started = Instant::now();
                    thread::sleep(Duration::from_millis(entry.delay_ms));
                    let _ = sender.send(Ok(provider::Answer {
                        proposal: entry.proposal,
                        latency_ms: started.elapsed().as_millis(),
                        usage: None,
                    }));
                })
            }
        };
        Ok(Pending {
            version,
            receiver,
            handle: Some(handle),
        })
    }
}
struct Pending {
    version: u64,
    receiver: Receiver<Result<provider::Answer, String>>,
    handle: Option<JoinHandle<()>>,
}
impl Drop for Pending {
    fn drop(&mut self) {
        // Never abandon a provider thread/curl child. The adapter's 30-second
        // timeout bounds this join; shutdown does not cancel in-flight inference.
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
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
    let mut source = if let Some(path) = replay {
        let entries: Vec<Replay> = serde_json::from_str(&config::bounded_file(path, 16384)?)
            .map_err(|_| "invalid_replay_sequence")?;
        if entries.is_empty()
            || entries.len() > coordinator::MAX_CALLS
            || entries.iter().any(|e| e.delay_ms > 2000)
        {
            return Err("invalid_replay_limits".into());
        }
        Source::Replay(entries.into())
    } else {
        Source::Meta(Arc::new(config::load()?))
    };
    println!(
        "{}",
        json!({"event":"coordinator_started","source":source.name(),"model":if matches!(source,Source::Meta(_)){Some(provider::MODEL)}else{None},"event_source":"scripted_release_fixture","goal":coordinator::GOAL,"max_model_calls":coordinator::MAX_CALLS})
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
            match waiting.receiver.try_recv() {
                Ok(answer) => Some((waiting.version, answer)),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    return Err("provider_thread_disconnected".into());
                }
            }
        } else {
            None
        };
        if let Some((version, answer)) = response {
            drop(pending.take());
            let answer = answer?;
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
            pending = Some(source.start(snapshot)?);
        }
        if cursor == scenario.events.len() && pending.is_none() && state.pending().is_none() {
            if let Source::Replay(entries) = &source
                && !entries.is_empty()
            {
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

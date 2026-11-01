//! Bounded asynchronous provider/replay transport shared by the two bridge modes.
use crate::{
    config::{self, Secret},
    domain::{self, Proposal},
    provider,
};
use serde::Deserialize;
use std::{
    collections::VecDeque,
    path::Path,
    sync::{
        Arc,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::Duration,
    time::Instant,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replay<P = Proposal> {
    delay_ms: u64,
    proposal: P,
}
pub enum Source<P = Proposal> {
    Meta(Arc<Secret>),
    Replay(VecDeque<Replay<P>>),
}
impl<P> Source<P> {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Meta(_) => "meta",
            Self::Replay(_) => "replay",
        }
    }
}
impl Source {
    pub fn start(
        &mut self,
        goal: &str,
        context: domain::DecisionContext,
    ) -> Result<Pending, String> {
        self.start_with(goal, context, provider::propose_context)
    }
}
impl Source<crate::batch::Plan> {
    pub fn start_batch(
        &mut self,
        goal: &str,
        context: domain::DecisionContext,
    ) -> Result<Pending<crate::batch::Plan>, String> {
        self.start_with(goal, context, provider::propose_batch)
    }
}
impl<P: Send + 'static> Source<P> {
    fn start_with(
        &mut self,
        goal: &str,
        context: domain::DecisionContext,
        propose: fn(
            &Secret,
            &str,
            &domain::DecisionContext,
        ) -> Result<provider::Answer<P>, provider::Failure>,
    ) -> Result<Pending<P>, String> {
        let version = context.state_version;
        let goal = goal.to_owned();
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = match self {
            Self::Meta(key) => {
                let key = Arc::clone(key);
                thread::spawn(move || {
                    let _ = sender.send(propose(&key, &goal, &context));
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
                        diagnostic: None,
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
pub struct Pending<P = Proposal> {
    pub version: u64,
    receiver: Receiver<Result<provider::Answer<P>, provider::Failure>>,
    handle: Option<JoinHandle<()>>,
}
impl<P> Drop for Pending<P> {
    fn drop(&mut self) {
        // Never abandon a provider thread/curl child. The adapter's 30-second
        // timeout bounds this join; shutdown does not cancel in-flight inference.
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
impl<P: serde::de::DeserializeOwned> Source<P> {
    pub fn load(replay: Option<&Path>, max_calls: usize) -> Result<Self, String> {
        if let Some(path) = replay {
            let entries: Vec<Replay<P>> = serde_json::from_str(&config::bounded_file(path, 16384)?)
                .map_err(|_| "invalid_replay_sequence")?;
            if entries.is_empty()
                || entries.len() > max_calls
                || entries.iter().any(|e| e.delay_ms > 2000)
            {
                return Err("invalid_replay_limits".into());
            }
            Ok(Self::Replay(entries.into()))
        } else {
            Ok(Self::Meta(Arc::new(config::load()?)))
        }
    }
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Meta(_))
    }
    pub fn has_unused(&self) -> bool {
        matches!(self, Self::Replay(entries) if !entries.is_empty())
    }
}
impl<P> Pending<P> {
    pub fn poll(&self) -> Result<Option<provider::Answer<P>>, String> {
        match self.receiver.try_recv() {
            Ok(answer) => report(answer, self.version).map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err("provider_thread_disconnected".into()),
        }
    }
}

/// One redacted diagnostic per consumed live provider outcome. Legacy response
/// events remain summaries of that same call; replay never fabricates diagnostics.
pub fn report<P>(
    result: Result<provider::Answer<P>, provider::Failure>,
    version: u64,
) -> Result<provider::Answer<P>, String> {
    match result {
        Ok(answer) => {
            if let Some(diagnostic) = &answer.diagnostic {
                println!(
                    "{}",
                    serde_json::json!({"event":"provider_diagnostic", "source":"meta",
                    "state_version":version, "outcome":"decoded", "error":null, "diagnostic":diagnostic})
                );
            }
            Ok(answer)
        }
        Err(failure) => {
            println!(
                "{}",
                serde_json::json!({"event":"provider_diagnostic", "source":"meta",
                "state_version":version, "outcome":"rejected", "error":failure.code,
                "diagnostic":failure.diagnostic})
            );
            Err(failure.code)
        }
    }
}

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
pub struct Replay {
    delay_ms: u64,
    proposal: Proposal,
}
pub enum Source {
    Meta(Arc<Secret>),
    Replay(VecDeque<Replay>),
}
impl Source {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Meta(_) => "meta",
            Self::Replay(_) => "replay",
        }
    }
    pub fn start(
        &mut self,
        goal: &str,
        context: domain::DecisionContext,
    ) -> Result<Pending, String> {
        let version = context.state_version;
        let goal = goal.to_owned();
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = match self {
            Self::Meta(key) => {
                let key = Arc::clone(key);
                thread::spawn(move || {
                    let _ = sender.send(provider::propose_context(&key, &goal, &context));
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
pub struct Pending {
    pub version: u64,
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
impl Source {
    pub fn load(replay: Option<&Path>, max_calls: usize) -> Result<Self, String> {
        if let Some(path) = replay {
            let entries: Vec<Replay> = serde_json::from_str(&config::bounded_file(path, 16384)?)
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
impl Pending {
    pub fn poll(&self) -> Result<Option<provider::Answer>, String> {
        match self.receiver.try_recv() {
            Ok(answer) => answer.map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err("provider_thread_disconnected".into()),
        }
    }
}

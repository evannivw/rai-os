//! One bounded cooperative job. Pure state transitions; no UART, clock, or heap.
use core::fmt;

pub const CAPACITY: usize = 192;
pub const CHUNK_BYTES: usize = 8;
pub const STEP_MS: u64 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Running,
    Completed,
    Cancelled,
}

impl State {
    pub fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub id: u32,
    pub state: State,
    pub processed: usize,
    pub total: usize,
    pub words: usize,
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "task_id={}; state={}; processed={}; total={}; words={}",
            self.id,
            self.state.name(),
            self.processed,
            self.total,
            self.words
        )
    }
}

pub struct Task {
    source: [u8; CAPACITY],
    snapshot: Snapshot,
    in_word: bool,
    next_step: u64,
}

impl Task {
    pub fn new() -> Self {
        Self {
            source: [0; CAPACITY],
            snapshot: Snapshot {
                id: 0,
                state: State::Idle,
                processed: 0,
                total: 0,
                words: 0,
            },
            in_word: false,
            next_step: 0,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot
    }

    pub fn start(&mut self, text: &str, now: u64) -> Result<Snapshot, &'static str> {
        if text.len() > CAPACITY {
            return Err("task_too_large");
        }
        if !text.bytes().all(|b| b.is_ascii_graphic() || b == b' ') {
            return Err("invalid_task_text");
        }
        if self.snapshot.state == State::Running {
            return Err("task_busy");
        }
        let id = self
            .snapshot
            .id
            .checked_add(1)
            .ok_or("task_ids_exhausted")?;
        self.source[..text.len()].copy_from_slice(text.as_bytes());
        self.snapshot = Snapshot {
            id,
            state: if text.is_empty() {
                State::Completed
            } else {
                State::Running
            },
            processed: 0,
            total: text.len(),
            words: 0,
        };
        self.in_word = false;
        self.next_step = now.saturating_add(STEP_MS);
        Ok(self.snapshot)
    }

    pub fn cancel(&mut self, id: u32) -> Result<Snapshot, &'static str> {
        if id == 0 || id != self.snapshot.id {
            return Err("stale_task");
        }
        if self.snapshot.state != State::Running {
            return Err("task_not_running");
        }
        self.snapshot.state = State::Cancelled;
        Ok(self.snapshot)
    }

    pub fn step(&mut self, now: u64) {
        if self.snapshot.state != State::Running || now < self.next_step {
            return;
        }
        let end = (self.snapshot.processed + CHUNK_BYTES).min(self.snapshot.total);
        for byte in &self.source[self.snapshot.processed..end] {
            let word = byte.is_ascii_alphanumeric();
            if word && !self.in_word {
                self.snapshot.words += 1;
            }
            self.in_word = word;
        }
        self.snapshot.processed = end;
        if end == self.snapshot.total {
            self.snapshot.state = State::Completed;
        }
        // Educational pacing, not a throughput benchmark. Never catch up by
        // processing multiple chunks in one loop after a delayed UART response.
        self.next_step = now.saturating_add(STEP_MS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn words_cross_chunks_and_punctuation() {
        let mut task = Task::new();
        task.start("abcdefghijk 12, abc-def!", 10).unwrap();
        task.step(109);
        assert_eq!(task.snapshot().processed, 0);
        task.step(110);
        assert_eq!((task.snapshot().processed, task.snapshot().words), (8, 1));
        task.step(110);
        assert_eq!(task.snapshot().processed, 8);
        task.step(10000);
        assert_eq!(task.snapshot().processed, 16); // No catch-up burst.
        task.step(10100);
        assert_eq!(task.snapshot().state, State::Completed);
        assert_eq!(task.snapshot().words, 4);
    }
    #[test]
    fn cancellation_and_stale_ids_preserve_state() {
        let mut task = Task::new();
        task.start("long enough to cancel", 0).unwrap();
        task.step(100);
        let before = task.snapshot();
        assert_eq!(task.start("replace", 100), Err("task_busy"));
        assert_eq!(task.cancel(2), Err("stale_task"));
        assert_eq!(task.snapshot(), before);
        let cancelled = task.cancel(1).unwrap();
        task.step(100000);
        assert_eq!(task.snapshot(), cancelled);
        assert_eq!(task.cancel(1), Err("task_not_running"));
        assert_eq!(task.start("next", 100000).unwrap().id, 2);
        assert_eq!(task.cancel(1), Err("stale_task"));
    }
    #[test]
    fn empty_limits_and_id_exhaustion() {
        let mut task = Task::new();
        assert_eq!(task.start("", 0).unwrap().state, State::Completed);
        assert_eq!(task.start("\n", 0), Err("invalid_task_text"));
        assert_eq!(
            task.start(&"x".repeat(CAPACITY + 1), 0),
            Err("task_too_large")
        );
        assert_eq!(task.snapshot().id, 1);
        task.start(&"x".repeat(CAPACITY), 0).unwrap();
        for step in 1..=CAPACITY / CHUNK_BYTES {
            task.step(step as u64 * STEP_MS);
        }
        assert_eq!(
            (task.snapshot().state, task.snapshot().words),
            (State::Completed, 1)
        );
        task.snapshot.id = u32::MAX;
        assert_eq!(task.start("next", 3000), Err("task_ids_exhausted"));
    }
}

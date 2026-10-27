//! Bounded FIFO policy. No threads, UART, UI, or model provider dependencies.
use crate::task_status::{State, TaskStatus};

pub const CAPACITY: usize = 8;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobState {
    Pending,
    Starting,
    Running,
    Completed,
    Cancelled,
    Interrupted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub text: String,
    pub state: JobState,
    pub observed: Option<TaskStatus>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Coordinator {
    pub jobs: Vec<Job>,
    pub enabled: bool,
}
impl Coordinator {
    pub fn busy(&self) -> bool {
        self.enabled || self.active().is_some()
    }
    fn active(&self) -> Option<usize> {
        self.jobs
            .iter()
            .position(|job| matches!(job.state, JobState::Starting | JobState::Running))
    }
    pub fn add(&mut self, text: String) -> Result<(), String> {
        if self.jobs.len() == CAPACITY {
            return Err("Queue full: clear finished jobs first".into());
        }
        if text.len() > 192 || !text.bytes().all(|b| (32..=126).contains(&b)) {
            return Err("Jobs require printable ASCII, at most 192 bytes".into());
        }
        self.jobs.push(Job {
            text,
            state: JobState::Pending,
            observed: None,
        });
        Ok(())
    }
    pub fn resume(&mut self, guest: TaskStatus) -> Result<(), String> {
        if self.active().is_none() && guest.state == State::Running {
            return Err("A manual task is running; wait for it to finish".into());
        }
        if !self.jobs.iter().any(|j| j.state == JobState::Pending) {
            return Err("Add a pending job first".into());
        }
        self.enabled = true;
        Ok(())
    }
    pub fn pause(&mut self) {
        self.enabled = false;
    }
    pub fn clear(&mut self) -> Result<(), String> {
        if self.busy() {
            return Err("Pause and wait for the current job before clearing".into());
        }
        self.jobs.clear();
        Ok(())
    }
    /// Reserve before sending: even a missing acknowledgement must never replay this job.
    pub fn take_next(&mut self) -> Option<String> {
        if !self.enabled || self.active().is_some() {
            return None;
        }
        if let Some(job) = self.jobs.iter_mut().find(|j| j.state == JobState::Pending) {
            job.state = JobState::Starting;
            Some(job.text.clone())
        } else {
            self.enabled = false;
            None
        }
    }
    pub fn observe(&mut self, task: TaskStatus) -> Result<(), String> {
        let Some(index) = self.active() else {
            return Ok(());
        };
        let job = &mut self.jobs[index];
        if task.state == State::Idle
            || task.total != job.text.len()
            || job.observed.is_some_and(|old| {
                old.id != task.id || task.processed < old.processed || task.words < old.words
            })
        {
            self.interrupt();
            return Err("Queue lost matching guest task; paused without retry".into());
        }
        job.observed = Some(task);
        job.state = match task.state {
            State::Running => JobState::Running,
            State::Completed => JobState::Completed,
            State::Cancelled => {
                self.enabled = false;
                JobState::Cancelled
            }
            State::Idle => unreachable!(),
        };
        Ok(())
    }
    pub fn interrupt(&mut self) {
        self.enabled = false;
        if let Some(index) = self.active() {
            self.jobs[index].state = JobState::Interrupted;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn task(id: u32, state: State, processed: usize, total: usize) -> TaskStatus {
        TaskStatus {
            id,
            state,
            processed,
            total,
            words: 0,
        }
    }
    fn queue() -> Coordinator {
        let mut q = Coordinator::default();
        q.add("first".into()).unwrap();
        q.add("second".into()).unwrap();
        q.resume(task(0, State::Idle, 0, 0)).unwrap();
        q
    }
    #[test]
    fn fifo_requires_observed_completion() {
        let mut q = queue();
        assert_eq!(q.take_next().as_deref(), Some("first"));
        assert!(q.take_next().is_none());
        q.observe(task(1, State::Running, 1, 5)).unwrap();
        assert!(q.take_next().is_none());
        q.observe(task(1, State::Completed, 5, 5)).unwrap();
        assert_eq!(q.take_next().as_deref(), Some("second"));
    }
    #[test]
    fn pause_drains_active_and_mismatch_never_advances() {
        let mut q = queue();
        q.take_next();
        q.observe(task(1, State::Running, 0, 5)).unwrap();
        q.pause();
        assert!(q.clear().is_err());
        q.observe(task(1, State::Completed, 5, 5)).unwrap();
        assert!(q.take_next().is_none());
        q.resume(task(1, State::Completed, 5, 5)).unwrap();
        q.take_next();
        q.observe(task(2, State::Running, 0, 6)).unwrap();
        assert!(q.observe(task(3, State::Completed, 6, 6)).is_err());
        assert_eq!(q.jobs[1].state, JobState::Interrupted);
        assert!(!q.busy());
    }
    #[test]
    fn disconnect_does_not_retry_unacknowledged_job() {
        let mut q = queue();
        q.take_next();
        q.interrupt();
        assert_eq!(q.jobs[0].state, JobState::Interrupted);
        assert!(q.take_next().is_none());
        q.resume(task(0, State::Idle, 0, 0)).unwrap();
        assert_eq!(q.take_next().as_deref(), Some("second"));
    }
    #[test]
    fn limits_manual_ownership_and_empty_completion() {
        let mut q = Coordinator::default();
        assert!(q.add("x".repeat(193)).is_err());
        assert!(q.add("tab\t".into()).is_err());
        for _ in 0..CAPACITY {
            q.add(String::new()).unwrap();
        }
        assert!(q.add(String::new()).is_err());
        assert!(q.resume(task(1, State::Running, 0, 5)).is_err());
        q.resume(task(1, State::Completed, 5, 5)).unwrap();
        for id in 2..10 {
            assert_eq!(q.take_next(), Some(String::new()));
            q.observe(task(id, State::Completed, 0, 0)).unwrap();
        }
        assert!(q.take_next().is_none());
        assert!(!q.busy());
        q.clear().unwrap();
    }
}

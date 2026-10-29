//! Pure fixed-capacity lifecycle rules; no registers, pointers or hardware access.
pub const MAX_SLICES: u8 = 6;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Ready,
    Running,
    Exited,
    Faulted,
    LimitReached,
}
impl State {
    pub fn name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Exited => "exited",
            Self::Faulted => "faulted",
            Self::LimitReached => "limit_reached",
        }
    }
}
#[derive(Clone, Copy)]
pub enum Event {
    Preempt,
    Exit,
    Fault,
}
pub struct Lifecycle {
    pub state: State,
    pub slices: u8,
}
impl Lifecycle {
    pub fn new() -> Self {
        Self {
            state: State::Ready,
            slices: 0,
        }
    }
    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.state != State::Ready {
            return Err("schedule_bad_transition");
        }
        self.state = State::Running;
        Ok(())
    }
    pub fn finish(&mut self, event: Event) -> Result<(), &'static str> {
        if self.state != State::Running {
            return Err("schedule_bad_transition");
        }
        self.state = match event {
            Event::Preempt => {
                self.slices += 1;
                if self.slices == MAX_SLICES {
                    State::LimitReached
                } else {
                    State::Ready
                }
            }
            Event::Exit => State::Exited,
            Event::Fault => State::Faulted,
        };
        Ok(())
    }
}
/// Cursor is a kernel-owned index in 0..2. Skip all non-ready entries.
pub fn next_ready(states: [State; 2], cursor: usize) -> Option<usize> {
    (0..2)
        .map(|offset| (cursor + offset) % 2)
        .find(|id| states[*id] == State::Ready)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exits_and_faults_are_terminal_and_do_not_stop_the_peer() {
        for event in [Event::Exit, Event::Fault] {
            let mut a = Lifecycle::new();
            let b = Lifecycle::new();
            assert!(a.finish(event).is_err());
            a.start().unwrap();
            assert!(a.start().is_err());
            a.finish(event).unwrap();
            assert_eq!(next_ready([a.state, b.state], 0), Some(1));
            let terminal = a.state;
            assert!(a.start().is_err());
            assert!(a.finish(Event::Preempt).is_err());
            assert_eq!(a.state, terminal);
            assert_eq!(b.state, State::Ready);
            assert_eq!(a.slices, 0);
        }
    }
    #[test]
    fn preemption_requeues_until_the_exact_limit_then_stays_terminal() {
        let mut task = Lifecycle::new();
        for i in 1..=MAX_SLICES {
            task.start().unwrap();
            task.finish(Event::Preempt).unwrap();
            assert_eq!(task.slices, i);
            assert_eq!(
                task.state,
                if i == MAX_SLICES {
                    State::LimitReached
                } else {
                    State::Ready
                }
            );
        }
        assert!(task.start().is_err());
        assert!(task.finish(Event::Exit).is_err());
        assert_eq!(task.slices, MAX_SLICES);
        assert_eq!(next_ready([task.state, State::Exited], 0), None);
    }
    #[test]
    fn alternation_skips_non_ready_slots_without_restarting_terminal_tasks() {
        assert_eq!(next_ready([State::Ready, State::Ready], 0), Some(0));
        assert_eq!(next_ready([State::Ready, State::Ready], 1), Some(1));
        assert_eq!(next_ready([State::Running, State::Ready], 0), Some(1));
        assert_eq!(next_ready([State::Faulted, State::Ready], 0), Some(1));
        assert_eq!(next_ready([State::Ready, State::Exited], 1), Some(0));
        assert_eq!(next_ready([State::Faulted, State::LimitReached], 1), None);
        assert_eq!(State::Ready.name(), "ready");
    }
}

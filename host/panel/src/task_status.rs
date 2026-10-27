//! Validate observed task snapshots before presenting progress. No job execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Running,
    Completed,
    Cancelled,
}
impl State {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskStatus {
    pub id: u32,
    pub state: State,
    pub processed: usize,
    pub total: usize,
    pub words: usize,
}
impl TaskStatus {
    pub fn parse(payload: &str) -> Result<Self, String> {
        let fields: Vec<_> = payload.split("; ").collect();
        if fields.len() != 5 {
            return Err("Invalid task field count".into());
        }
        fn number(text: &str, prefix: &str) -> Result<usize, String> {
            let digits = text.strip_prefix(prefix).ok_or("Invalid task field name")?;
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return Err("Invalid task digits".into());
            }
            digits.parse().map_err(|_| "Task number overflow".into())
        }
        let value = Self {
            id: u32::try_from(number(fields[0], "task_id=")?).map_err(|_| "Task ID overflow")?,
            state: match fields[1] {
                "state=idle" => State::Idle,
                "state=running" => State::Running,
                "state=completed" => State::Completed,
                "state=cancelled" => State::Cancelled,
                _ => return Err("Unknown task state".into()),
            },
            processed: number(fields[2], "processed=")?,
            total: number(fields[3], "total=")?,
            words: number(fields[4], "words=")?,
        };
        if value.total > 192
            || value.processed > value.total
            || value.words > value.processed.div_ceil(2)
        {
            return Err("Invalid task counters".into());
        }
        let valid = match value.state {
            State::Idle => value.id == 0 && value.total == 0,
            State::Completed => value.id != 0 && value.processed == value.total,
            State::Running | State::Cancelled => value.id != 0 && value.processed < value.total,
        };
        if !valid {
            return Err("Inconsistent task state".into());
        }
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_inconsistent_task_snapshots() {
        for payload in [
            "task_id=1; state=completed; processed=1; total=10; words=1",
            "task_id=0; state=running; processed=1; total=10; words=1",
            "task_id=1; state=running; processed=2; total=1; words=1",
            "task_id=1; state=running; processed=1; total=193; words=1",
            "task_id=1; state=cancelled; processed=8; total=10; words=8",
            "task_id=1; state=completed; processed=0; total=0; words=0; extra=1",
        ] {
            assert!(TaskStatus::parse(payload).is_err(), "{payload}");
        }
        assert!(
            TaskStatus::parse("task_id=1; state=completed; processed=0; total=0; words=0").is_ok()
        );
    }
}

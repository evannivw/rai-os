//! Bounded scalar mailboxes. Pure policy; no pointers, registers, UART or MMU.
pub const TASK_COUNT: usize = 2;
pub const CAPACITY: usize = 2;
pub const MAX_CALLS: u8 = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskId(usize);
impl TaskId {
    pub fn from_raw(raw: u64) -> Result<Self, Error> {
        if raw == 0 || raw > TASK_COUNT as u64 {
            return Err(Error::InvalidTask);
        }
        Ok(Self(raw as usize - 1))
    }
    pub fn raw(self) -> u64 {
        self.0 as u64 + 1
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Error {
    InvalidTask = 1,
    Denied = 2,
    Full = 3,
    Empty = 4,
    Closed = 5,
    InvalidKind = 6,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    pub sender: TaskId,
    pub kind: u8,
    pub value: u64,
}
struct Mailbox {
    entries: [Option<Message>; CAPACITY],
    head: usize,
    len: usize,
}
impl Mailbox {
    fn new() -> Self {
        Self {
            entries: [None; CAPACITY],
            head: 0,
            len: 0,
        }
    }
}
pub struct Mailboxes {
    boxes: [Mailbox; TASK_COUNT],
    live: [bool; TASK_COUNT],
    routes: [u8; TASK_COUNT],
}
impl Mailboxes {
    /// Routes are kernel configuration, not user input. Bit n grants send to ID n+1.
    pub fn new(routes: [u8; TASK_COUNT]) -> Self {
        Self {
            boxes: core::array::from_fn(|_| Mailbox::new()),
            live: [true; TASK_COUNT],
            routes,
        }
    }
    /// Caller identity comes from the selected kernel task record, never a syscall operand.
    pub fn send(
        &mut self,
        caller: TaskId,
        destination: u64,
        kind: u64,
        value: u64,
    ) -> Result<(), Error> {
        let target = TaskId::from_raw(destination)?;
        if !self.live[caller.0] || !self.live[target.0] {
            return Err(Error::Closed);
        }
        if self.routes[caller.0] & (1 << target.0) == 0 {
            return Err(Error::Denied);
        }
        if kind == 0 || kind > u8::MAX as u64 {
            return Err(Error::InvalidKind);
        }
        let inbox = &mut self.boxes[target.0];
        if inbox.len == CAPACITY {
            return Err(Error::Full);
        }
        inbox.entries[(inbox.head + inbox.len) % CAPACITY] = Some(Message {
            sender: caller,
            kind: kind as u8,
            value,
        });
        inbox.len += 1;
        Ok(())
    }
    /// No destination operand: a task can receive only from its own inbox.
    pub fn receive(&mut self, caller: TaskId) -> Result<Message, Error> {
        if !self.live[caller.0] {
            return Err(Error::Closed);
        }
        let inbox = &mut self.boxes[caller.0];
        if inbox.len == 0 {
            return Err(Error::Empty);
        }
        let message = inbox.entries[inbox.head]
            .take()
            .expect("occupied mailbox slot");
        inbox.head = (inbox.head + 1) % CAPACITY;
        inbox.len -= 1;
        Ok(message)
    }
    pub fn close(&mut self, task: TaskId) {
        self.live[task.0] = false;
        self.boxes[task.0] = Mailbox::new();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn id(raw: u64) -> TaskId {
        TaskId::from_raw(raw).unwrap()
    }
    #[test]
    fn fifo_full_rejection_and_wrap_preserve_accepted_messages() {
        let mut boxes = Mailboxes::new([0b10, 0b01]);
        boxes.send(id(1), 2, 1, 7).unwrap();
        boxes.send(id(1), 2, 2, u64::MAX).unwrap();
        assert_eq!(boxes.send(id(1), 2, 3, 99), Err(Error::Full));
        assert_eq!(
            boxes.receive(id(2)),
            Ok(Message {
                sender: id(1),
                kind: 1,
                value: 7
            })
        );
        boxes.send(id(1), 2, 3, 42).unwrap();
        assert_eq!(boxes.receive(id(2)).unwrap().value, u64::MAX);
        assert_eq!(boxes.receive(id(2)).unwrap().value, 42);
        assert_eq!(boxes.receive(id(2)), Err(Error::Empty));
    }
    #[test]
    fn identity_routes_and_invalid_inputs_do_not_mutate_mailboxes() {
        let mut boxes = Mailboxes::new([0b10, 0]);
        for raw in [0, 3, u64::MAX] {
            assert_eq!(TaskId::from_raw(raw), Err(Error::InvalidTask));
            assert_eq!(boxes.send(id(1), raw, 1, 0), Err(Error::InvalidTask));
        }
        assert_eq!(boxes.send(id(1), 1, 1, 0), Err(Error::Denied));
        assert_eq!(boxes.send(id(2), 1, 1, 0), Err(Error::Denied));
        for kind in [0, 256, u64::MAX] {
            assert_eq!(boxes.send(id(1), 2, kind, 0), Err(Error::InvalidKind));
        }
        for raw in [1, 2] {
            assert_eq!(boxes.receive(id(raw)), Err(Error::Empty));
        }
        boxes.send(id(1), 2, 255, 17).unwrap();
        assert_eq!(
            boxes.receive(id(2)),
            Ok(Message {
                sender: id(1),
                kind: 255,
                value: 17
            })
        );
        assert_eq!(id(2).raw(), 2);
    }
    #[test]
    fn closing_discards_own_queue_but_preserves_already_sent_messages() {
        let mut boxes = Mailboxes::new([0b10, 0b01]);
        boxes.send(id(1), 2, 1, 7).unwrap();
        boxes.send(id(2), 1, 2, 8).unwrap();
        boxes.close(id(1));
        boxes.close(id(1));
        assert_eq!(boxes.receive(id(1)), Err(Error::Closed));
        assert_eq!(boxes.send(id(2), 1, 1, 0), Err(Error::Closed));
        assert_eq!(boxes.send(id(1), 2, 1, 0), Err(Error::Closed));
        assert_eq!(boxes.receive(id(2)).unwrap().value, 7);
        assert_eq!(boxes.receive(id(2)), Err(Error::Empty));
        let mut fresh = Mailboxes::new([0b10, 0b01]);
        assert_eq!(fresh.receive(id(1)), Err(Error::Empty));
        assert_eq!(fresh.receive(id(2)), Err(Error::Empty));
    }
}

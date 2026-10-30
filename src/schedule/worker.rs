//! One persistent guest worker. Host ingress is endpoint 1; EL0 worker is endpoint 2.
//! Kernel-owned saved state and mailboxes outlive individual console commands.
use super::{run_turn, runtime::valid_context};
use crate::{
    context::Frame,
    ipc::{self, Mailboxes, Message, TaskId},
    mmu::{Mmu, TaskSpace},
    records::RecordId,
    timer::Timer,
    user_memory::Lease,
};
use core::{fmt, ptr};
const MAX_SLICES: u8 = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Stopped,
    Ready,
    Waiting,
    Failed,
}
impl State {
    fn name(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Ready => "ready",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
        }
    }
}
pub struct Worker {
    lease: Option<Lease>,
    frame: Frame,
    boxes: Mailboxes,
    state: State,
    reason: &'static str,
    completed: u64,
    calls: u8,
    slices: u8,
}
#[derive(Clone, Copy)]
pub struct Status {
    state: State,
    reason: &'static str,
    completed: u64,
    calls: u8,
    slices: u8,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "worker=EL0; state={}; completed={}; reason={}; calls={}; slices={}",
            self.state.name(),
            self.completed,
            self.reason,
            self.calls,
            self.slices
        )
    }
}
pub struct Count {
    id: RecordId,
    packed: u64,
    status: Status,
}
impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "record={}; bytes={}; tokens={}; {}",
            self.id,
            (self.packed >> 16) & 0xffff,
            self.packed & 0xffff,
            self.status
        )
    }
}
fn endpoint(id: u64) -> TaskId {
    TaskId::from_raw(id).expect("fixed endpoint")
}
fn bounds() -> ((u64, u64), (u64, u64)) {
    unsafe extern "C" {
        static mut __user_stack_bottom: [u8; 8192];
        static __user_text_start: u8;
        static __user_text_end: u8;
    }
    let bottom = ptr::addr_of_mut!(__user_stack_bottom) as u64;
    (
        (bottom + 4096, bottom + 8192),
        (
            ptr::addr_of!(__user_text_start) as u64,
            ptr::addr_of!(__user_text_end) as u64,
        ),
    )
}
impl Worker {
    pub fn new() -> Self {
        Self {
            lease: None,
            frame: Frame::default(),
            boxes: Mailboxes::new([0b10, 0b01]),
            state: State::Stopped,
            reason: "none",
            completed: 0,
            calls: 0,
            slices: 0,
        }
    }
    pub fn status(&self) -> Status {
        Status {
            state: self.state,
            reason: self.reason,
            completed: self.completed,
            calls: self.calls,
            slices: self.slices,
        }
    }
    fn clear_owned_memory(&mut self) {
        if self.lease.is_some() {
            let (stack, _) = bounds();
            // SAFETY: lease excludes all other EL0 users of the whole 8 KiB region.
            // Worker is paused/retired, base map active, one masked CPU, no DMA.
            unsafe {
                ptr::write_bytes((stack.0 - 4096) as *mut u8, 0, 8192);
            }
        }
        // Retire the saved continuation before relinquishing memory ownership.
        self.frame = Frame::default();
        self.boxes = Mailboxes::new([0b10, 0b01]);
        self.lease = None;
    }
    pub fn stop(&mut self) -> Status {
        self.clear_owned_memory();
        self.state = State::Stopped;
        self.reason = "none";
        self.completed = 0;
        self.calls = 0;
        self.slices = 0;
        self.status()
    }
    fn fail(&mut self, reason: &'static str) -> &'static str {
        self.clear_owned_memory();
        self.state = State::Failed;
        self.reason = reason;
        reason
    }
    pub fn start(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<Status, &'static str> {
        if self.state != State::Stopped {
            return Err("worker_not_stopped");
        }
        let lease = Lease::acquire()?;
        self.lease = Some(lease);
        let (stack, _) = bounds();
        // SAFETY: fresh exclusive lease; no live user context. Clear before entry.
        unsafe {
            ptr::write_bytes((stack.0 - 4096) as *mut u8, 0, 8192);
        }
        self.frame = Frame {
            elr: super::worker_program::persistent_worker as *const () as u64,
            user_sp: stack.1,
            spsr: 0x340,
            ..Frame::default()
        };
        self.state = State::Ready;
        if let Err(error) = self.pump(timer, mmu) {
            return Err(self.fail(error));
        }
        if self.boxes.receive(endpoint(1)) != Err(ipc::Error::Empty) {
            return Err(self.fail("worker_unexpected_reply"));
        }
        Ok(self.status())
    }
    pub fn count(
        &mut self,
        id: RecordId,
        timer: &mut Timer,
        mmu: &Mmu,
    ) -> Result<Count, &'static str> {
        if self.state != State::Waiting {
            return Err("worker_not_waiting");
        }
        let next = self
            .completed
            .checked_add(1)
            .ok_or("worker_sequence_limit")?;
        // Admission is synchronous: one request in flight, never a queued host backlog.
        if self.boxes.send(endpoint(1), 2, 20, id.raw()).is_err() {
            return Err(self.fail("worker_ingress_failed"));
        }
        let request = match self.boxes.receive(endpoint(2)) {
            Ok(request) => request,
            Err(_) => return Err(self.fail("worker_ingress_failed")),
        };
        // Complete the suspended receive, preserving PC/SP and all other registers.
        self.deliver(request);
        self.state = State::Ready;
        if let Err(error) = self.pump(timer, mmu) {
            return Err(self.fail(error));
        }
        let result = self.boxes.receive(endpoint(1));
        let sequence = self.boxes.receive(endpoint(1));
        let extra = self.boxes.receive(endpoint(1));
        let (Ok(result), Ok(sequence)) = (result, sequence) else {
            return Err(self.fail("worker_reply_rejected"));
        };
        if result.sender != endpoint(2)
            || result.kind != 21
            || result.value >> 32 != id.raw()
            || ((result.value >> 16) & 0xffff) != id.bytes()
            || (result.value & 0xffff) > id.bytes()
            || sequence.sender != endpoint(2)
            || sequence.kind != 23
            || sequence.value != next
            || extra != Err(ipc::Error::Empty)
        {
            return Err(self.fail("worker_reply_rejected"));
        }
        self.completed = next;
        Ok(Count {
            id,
            packed: result.value,
            status: self.status(),
        })
    }
    fn deliver(&mut self, message: Message) {
        self.frame.registers[..4].copy_from_slice(&[
            0,
            message.sender.raw(),
            message.kind as u64,
            message.value,
        ]);
    }
    fn pump(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<(), &'static str> {
        self.calls = 0;
        self.slices = 0;
        let (stack, text) = bounds();
        loop {
            if self.lease.is_none() || !valid_context(&self.frame, stack, text) {
                return Err("worker_context_invalid");
            }
            // SAFETY: lease retains exclusive user RAM across commands; context
            // validation checks executable aligned PC, private B SP and EL0 PSTATE.
            // Console runs on one masked CPU, with no other active invocation.
            // run_turn uses a separate output and restores the base map on return.
            let observed = unsafe { run_turn(timer, mmu, TaskSpace::from_index(1)?, &self.frame) }?;
            self.frame = observed.frame;
            if !valid_context(&self.frame, stack, text) {
                return Err("worker_context_invalid");
            }
            if observed.vector == 9 {
                self.slices += 1;
                if self.slices == MAX_SLICES {
                    return Err("worker_slice_limit");
                }
                continue;
            }
            if observed.vector != 8 || self.frame.esr != 0x5600_0000 {
                return Err("worker_fault");
            }
            self.calls += 1;
            if self.calls == ipc::MAX_CALLS {
                return Err("worker_call_limit");
            }
            let args = self.frame.registers;
            self.frame.registers[..4].fill(0);
            match args[8] {
                10 => {
                    if let Err(error) = self.boxes.send(endpoint(2), args[0], args[1], args[2]) {
                        self.frame.registers[0] = error as u64;
                    }
                }
                12 => match self.boxes.receive_wait(endpoint(2)) {
                    Ok(Some(message)) => self.deliver(message),
                    Ok(None) => {
                        self.state = State::Waiting;
                        return Ok(());
                    }
                    Err(error) => self.frame.registers[0] = error as u64,
                },
                _ => return Err("worker_bad_syscall"),
            }
        }
    }
}

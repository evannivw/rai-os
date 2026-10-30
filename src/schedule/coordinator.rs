//! Two retained EL0 services with coordinator-only host ingress/publication.
//! Transport, bounded dispatch and report validation; goal policy lives at EL0.
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
#[derive(Clone, Copy, PartialEq, Eq)]
enum Wait {
    Ready,
    Host,
    Ipc,
}
struct Task {
    frame: Frame,
    wait: Wait,
}
impl Task {
    fn new() -> Self {
        Self {
            frame: Frame::default(),
            wait: Wait::Ready,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Report {
    state: &'static str,
    event: &'static str,
    ticket: u64,
    result: u64,
    completed: u64,
    calls: [u8; 2],
    slices: [u8; 2],
    trace: [u8; 192],
    length: usize,
}
impl Report {
    fn stopped() -> Self {
        Self {
            state: "stopped",
            event: "none",
            ticket: 0,
            result: 0,
            completed: 0,
            calls: [0; 2],
            slices: [0; 2],
            trace: [0; 192],
            length: 0,
        }
    }
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "coordinator=EL0; worker=EL0; state={}; event={}; ticket={}; record=",
            self.state, self.event, self.ticket
        )?;
        if let Ok(id) = RecordId::from_raw(self.result >> 32) {
            write!(f, "{id}")?;
        } else {
            f.write_str("none")?;
        }
        write!(
            f,
            "; bytes={}; tokens={}; completed={}; a_calls={}; b_calls={}; a_slices={}; b_slices={}; trace=",
            (self.result >> 16) & 0xffff,
            self.result & 0xffff,
            self.completed,
            self.calls[0],
            self.calls[1],
            self.slices[0],
            self.slices[1]
        )?;
        for (i, event) in self.trace[..self.length].chunks_exact(2).enumerate() {
            if i != 0 {
                f.write_str(",")?;
            }
            f.write_str(core::str::from_utf8(event).map_err(|_| fmt::Error)?)?;
        }
        Ok(())
    }
}
/// Typed proposal boundary; tickets are nonzero u32 values, keys use the record allowlist.
pub struct Proposal {
    pub ticket: u32,
    pub id: RecordId,
}
pub struct Coordinator {
    lease: Option<Lease>,
    tasks: [Task; 2],
    boxes: Mailboxes,
    report: Report,
    // Monotonic across stop/start within this boot. Never reuse a delayed proposal ticket.
    issued: u32,
}
fn endpoint(id: usize) -> TaskId {
    TaskId::from_raw(id as u64 + 1).expect("fixed task")
}
fn bounds(id: usize) -> ((u64, u64), (u64, u64)) {
    unsafe extern "C" {
        static mut __user_stack_bottom: [u8; 8192];
        static __user_text_start: u8;
        static __user_text_end: u8;
    }
    let bottom = ptr::addr_of_mut!(__user_stack_bottom) as u64 + id as u64 * 4096;
    (
        (bottom, bottom + 4096),
        (
            ptr::addr_of!(__user_text_start) as u64,
            ptr::addr_of!(__user_text_end) as u64,
        ),
    )
}
impl Coordinator {
    pub fn new() -> Self {
        Self {
            lease: None,
            tasks: [Task::new(), Task::new()],
            boxes: Mailboxes::new([0b10, 0b01]),
            report: Report::stopped(),
            issued: 0,
        }
    }
    pub fn status(&self) -> Report {
        self.report
    }
    fn retire(&mut self) {
        if self.lease.is_some() {
            // SAFETY: exclusive lease over both stack pages; tasks suspended, base map
            // restored by run_turn; single masked CPU, no DMA or live user references.
            unsafe {
                ptr::write_bytes(bounds(0).0.0 as *mut u8, 0, 8192);
            }
        }
        self.tasks = [Task::new(), Task::new()];
        self.boxes = Mailboxes::new([0b10, 0b01]);
        self.lease = None;
    }
    pub fn stop(&mut self) -> Report {
        self.retire();
        self.report = Report::stopped();
        self.report
    }
    fn fail(&mut self, error: &'static str) -> &'static str {
        self.retire();
        self.report.state = "failed";
        self.report.event = error;
        error
    }
    pub fn start(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<Report, &'static str> {
        if self.report.state != "stopped" {
            return Err("coord_not_stopped");
        }
        self.lease = Some(Lease::acquire()?);
        // SAFETY: fresh exclusive lease, neither task entered; fixed linker-owned RAM.
        unsafe {
            ptr::write_bytes(bounds(0).0.0 as *mut u8, 0, 8192);
        }
        let entries = [
            super::coordinator_program::guest_coordinator as *const () as u64,
            super::worker_program::persistent_worker as *const () as u64,
        ];
        for (id, entry) in entries.into_iter().enumerate() {
            self.tasks[id].frame = Frame {
                elr: entry,
                user_sp: bounds(id).0.1,
                spsr: 0x340,
                ..Frame::default()
            };
        }
        self.finish_turn(timer, mmu)
    }
    pub fn observe(
        &mut self,
        id: RecordId,
        timer: &mut Timer,
        mmu: &Mmu,
    ) -> Result<Report, &'static str> {
        self.require_waiting()?;
        let ticket = self.issued.checked_add(1).ok_or("coord_ticket_limit")?;
        self.issued = ticket;
        self.input(30, ticket, id);
        self.finish_turn(timer, mmu)
    }
    pub fn propose(
        &mut self,
        proposal: Proposal,
        timer: &mut Timer,
        mmu: &Mmu,
    ) -> Result<Report, &'static str> {
        self.require_waiting()?;
        self.input(31, proposal.ticket, proposal.id);
        let report = self.finish_turn(timer, mmu)?;
        match report.event {
            "stale_proposal" => Err("coord_stale_proposal"),
            "wrong_record" => Err("coord_wrong_record"),
            "already_completed" => Err("coord_already_completed"),
            "no_goal" => Err("coord_no_goal"),
            _ => Ok(report),
        }
    }
    fn require_waiting(&self) -> Result<(), &'static str> {
        if self.lease.is_some()
            && self.tasks[0].wait == Wait::Host
            && self.tasks[1].wait == Wait::Ipc
        {
            Ok(())
        } else {
            Err("coord_not_running")
        }
    }
    fn input(&mut self, kind: u64, ticket: u32, id: RecordId) {
        self.tasks[0].frame.registers[..4].copy_from_slice(&[0, kind, ticket as u64, id.raw()]);
        self.tasks[0].wait = Wait::Ready;
    }
    fn trace(&mut self, id: usize, event: u8) -> Result<(), &'static str> {
        let n = self.report.length;
        if n + 2 > self.report.trace.len() {
            return Err("coord_trace_limit");
        }
        self.report.trace[n] = b'A' + id as u8;
        self.report.trace[n + 1] = event;
        self.report.length += 2;
        Ok(())
    }
    fn deliver(&mut self, id: usize, message: Message) {
        self.tasks[id].frame.registers[..4].copy_from_slice(&[
            0,
            message.sender.raw(),
            message.kind as u64,
            message.value,
        ]);
        self.tasks[id].wait = Wait::Ready;
    }
    fn finish_turn(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<Report, &'static str> {
        // Publish to the host only after BOTH continuations are waiting again.
        let previous = self.report;
        if let Err(error) = self.pump(timer, mmu) {
            let (calls, slices, trace, length) = (
                self.report.calls,
                self.report.slices,
                self.report.trace,
                self.report.length,
            );
            self.report = previous;
            self.report.calls = calls;
            self.report.slices = slices;
            self.report.trace = trace;
            self.report.length = length;
            return Err(self.fail(error));
        }
        Ok(self.report)
    }
    fn pump(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<(), &'static str> {
        self.report.calls = [0; 2];
        self.report.slices = [0; 2];
        self.report.length = 0;
        let mut cursor = 0;
        let mut publication = None;
        let mut worker_result = None;
        let mut worker_sequence = None;
        loop {
            let ready = [cursor, cursor ^ 1]
                .into_iter()
                .find(|id| self.tasks[*id].wait == Wait::Ready);
            let Some(id) = ready else {
                break;
            };
            let (stack, text) = bounds(id);
            if self.lease.is_none() || !valid_context(&self.tasks[id].frame, stack, text) {
                return Err("coord_context_invalid");
            }
            // SAFETY: lease owns both user pages across requests; validated EL0 PC,
            // private aligned SP/PSTATE; one masked kernel CPU. Each turn selects
            // its task map, captures to separate output, then restores the base map.
            let observed = unsafe {
                run_turn(
                    timer,
                    mmu,
                    TaskSpace::from_index(id)?,
                    &self.tasks[id].frame,
                )
            }?;
            self.tasks[id].frame = observed.frame;
            if !valid_context(&self.tasks[id].frame, stack, text) {
                return Err("coord_context_invalid");
            }
            if observed.vector == 9 {
                self.report.slices[id] += 1;
                self.trace(id, b'P')?;
                if self.report.slices[id] == 6 {
                    return Err("coord_slice_limit");
                }
            } else {
                if observed.vector != 8 || observed.frame.esr != 0x5600_0000 {
                    self.trace(id, b'F')?;
                    return Err("coord_task_fault");
                }
                self.report.calls[id] += 1;
                if self.report.calls[id] == ipc::MAX_CALLS {
                    self.trace(id, b'L')?;
                    return Err("coord_call_limit");
                }
                let args = self.tasks[id].frame.registers;
                self.tasks[id].frame.registers[..4].fill(0);
                match args[8] {
                    10 => {
                        match self.boxes.send(endpoint(id), args[0], args[1], args[2]) {
                            Ok(()) => {
                                if id == 1 && args[1] == 21 {
                                    worker_result = Some(args[2]);
                                }
                                if id == 1 && args[1] == 23 {
                                    worker_sequence = Some(args[2]);
                                }
                            }
                            Err(error) => self.tasks[id].frame.registers[0] = error as u64,
                        }
                        self.trace(id, b'S')?;
                    }
                    12 => match self.boxes.receive_wait(endpoint(id)) {
                        Ok(Some(message)) => {
                            self.deliver(id, message);
                            self.trace(id, b'R')?;
                        }
                        Ok(None) => {
                            self.tasks[id].wait = Wait::Ipc;
                            self.trace(id, b'D')?;
                        }
                        Err(_) => return Err("coord_ipc_closed"),
                    },
                    13 if id == 0 => {
                        self.tasks[id].wait = Wait::Host;
                        self.trace(id, b'H')?;
                    }
                    14 if id == 0 => {
                        if publication.is_some() {
                            return Err("coord_extra_report");
                        }
                        publication = Some([args[0], args[1], args[2], args[3], args[4]]);
                        self.trace(id, b'O')?;
                    }
                    _ => {
                        self.trace(id, b'F')?;
                        return Err("coord_bad_syscall");
                    }
                }
                for waiting in 0..2 {
                    if self.tasks[waiting].wait == Wait::Ipc {
                        match self.boxes.receive_wait(endpoint(waiting)) {
                            Ok(Some(message)) => {
                                self.deliver(waiting, message);
                                self.trace(waiting, b'W')?;
                            }
                            Ok(None) => {}
                            Err(_) => return Err("coord_ipc_closed"),
                        }
                    }
                }
            }
            cursor = id ^ 1;
        }
        if self.tasks[0].wait != Wait::Host || self.tasks[1].wait != Wait::Ipc {
            return Err("coord_deadlocked");
        }
        let [state, ticket, result, completed, event] =
            publication.ok_or("coord_missing_report")?;
        if state > 2 || event > 7 || ticket > self.issued as u64 || completed > self.issued as u64 {
            return Err("coord_bad_report");
        }
        if state == 0 {
            if ticket != 0 || result != 0 || completed != 0 {
                return Err("coord_bad_report");
            }
        } else {
            let id = RecordId::from_raw(result >> 32).map_err(|_| "coord_bad_report")?;
            if ticket == 0
                || (state == 1 && result != id.raw() << 32)
                || (state == 2
                    && (((result >> 16) & 0xffff) != id.bytes() || (result & 0xffff) > id.bytes()))
            {
                return Err("coord_bad_report");
            }
        }
        if event == 3 {
            if state != 2
                || worker_result != Some(result)
                || worker_sequence != Some(completed)
                || self.report.completed.checked_add(1) != Some(completed)
            {
                return Err("coord_bad_report");
            }
        } else if completed != self.report.completed
            || worker_result.is_some()
            || worker_sequence.is_some()
        {
            return Err("coord_bad_report");
        }
        // A non-executing event may repeat a confirmed result, never invent one.
        if state == 2
            && event != 3
            && (self.report.state != "completed"
                || result != self.report.result
                || ticket != self.report.ticket)
        {
            return Err("coord_bad_report");
        }
        self.report.state = ["idle", "awaiting_proposal", "completed"][state as usize];
        self.report.event = [
            "started",
            "observed",
            "duplicate",
            "executed",
            "stale_proposal",
            "wrong_record",
            "already_completed",
            "no_goal",
        ][event as usize];
        self.report.ticket = ticket;
        self.report.result = result;
        self.report.completed = completed;
        Ok(())
    }
}

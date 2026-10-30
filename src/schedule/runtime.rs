//! Two kernel-owned task records with terminal-state skipping and bounded lifetime.
use super::{
    is_active,
    lifecycle::{Event, Lifecycle, MAX_SLICES, State, next_ready},
    run_turn,
};
use crate::{
    context::Frame,
    ipc::{MAX_CALLS, Mailboxes, TaskId},
    mmu::{Mmu, TaskSpace},
    timer::{SLICE_MS, Timer},
};
use core::{fmt, ptr};
#[derive(Clone, Copy)]
pub enum RunMode {
    Finish,
    Fault,
    Messages,
    WaitMessages,
    RecordCount(crate::records::RecordId),
}
struct Record {
    frame: Frame,
    stack: (u64, u64),
    lifecycle: Lifecycle,
    reason: &'static str,
    code: u64,
    value: u64,
    calls: u8,
}
// One byte per event: high bit is task index, low seven bits are the ASCII code.
// Reserve dispatch + possible wake/cancellation events, with no heap allocation.
const MAX_EVENTS: usize = (MAX_SLICES as usize + MAX_CALLS as usize + 1) * 4;
#[derive(Default)]
struct IpcStats {
    sent: u8,
    received: u8,
    rejected: u8,
    empty: u8,
    calls: [u8; 2],
    blocked: u8,
    woken: u8,
    closed: u8,
    deadlocks: u8,
}
struct FaultInfo {
    esr: u64,
    elr: u64,
    far: u64,
    spsr: u64,
    user_sp: u64,
}
struct TaskResult {
    lifecycle: Lifecycle,
    reason: &'static str,
    code: u64,
    value: u64,
    fault: FaultInfo,
}
pub struct RunReport {
    records: [TaskResult; 2],
    trace: [u8; MAX_EVENTS],
    length: usize,
    ipc: Option<IpcStats>,
    waiting: bool,
}
impl RunReport {
    /// Exit codes/values only when both tasks actually exited (not fault/limit).
    pub fn exits(&self) -> Option<[(u64, u64); 2]> {
        let [a, b] = &self.records;
        (a.lifecycle.state == State::Exited && b.lifecycle.state == State::Exited)
            .then_some([(a.code, a.value), (b.code, b.value)])
    }
}
impl fmt::Display for RunReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tasks=2; slice_ms={SLICE_MS}; max_slices={MAX_SLICES}; trace="
        )?;
        for (i, event) in self.trace[..self.length].iter().enumerate() {
            if i != 0 {
                f.write_str(",")?;
            }
            write!(
                f,
                "{}{}",
                char::from(b'A' + (event >> 7)),
                char::from(event & 0x7f)
            )?;
        }
        for (id, task) in self.records.iter().enumerate() {
            let name = char::from(b'a' + id as u8);
            write!(
                f,
                "; {name}_state={}; {name}_slices={}",
                task.lifecycle.state.name(),
                task.lifecycle.slices
            )?;
            match task.lifecycle.state {
                State::Exited => write!(
                    f,
                    "; {name}_code={}; {name}_value={}",
                    task.code, task.value
                )?,
                State::Faulted => write!(
                    f,
                    "; {name}_reason={}; {name}_esr=0x{:016x}; {name}_pc=0x{:016x}; {name}_far_raw=0x{:016x}; {name}_spsr=0x{:016x}; {name}_sp=0x{:016x}",
                    task.reason,
                    task.fault.esr,
                    task.fault.elr,
                    task.fault.far,
                    task.fault.spsr,
                    task.fault.user_sp
                )?,
                _ => {}
            }
        }
        if let Some(ipc) = &self.ipc {
            write!(
                f,
                "; ipc_capacity=2; max_calls={MAX_CALLS}; sent={}; received={}; rejected={}; empty={}; a_calls={}; b_calls={}",
                ipc.sent, ipc.received, ipc.rejected, ipc.empty, ipc.calls[0], ipc.calls[1]
            )?;
            if self.waiting {
                write!(
                    f,
                    "; blocked={}; woken={}; closed_wakes={}; deadlocked={}",
                    ipc.blocked, ipc.woken, ipc.closed, ipc.deadlocks
                )?;
            }
        }
        Ok(())
    }
}
pub fn run(timer: &mut Timer, mmu: &Mmu, mode: RunMode) -> Result<RunReport, &'static str> {
    if is_active() || crate::user::is_active() {
        return Err("schedule_busy");
    }
    unsafe extern "C" {
        static mut __user_stack_bottom: [u8; 8192];
        static __user_text_start: u8;
        static __user_text_end: u8;
        fn record_client();
        fn record_worker();
        fn wait_program_a();
        fn wait_program_b();
        fn ipc_program_a();
        fn ipc_program_b();
        fn lifecycle_program_a();
        fn lifecycle_program_b();
        fn lifecycle_fault();
        fn user_spin();
    }
    let bottom = ptr::addr_of_mut!(__user_stack_bottom) as u64;
    let text = (
        ptr::addr_of!(__user_text_start) as u64,
        ptr::addr_of!(__user_text_end) as u64,
    );
    let spin_a = matches!(mode, RunMode::Finish)
        && cfg!(any(
            feature = "lifecycle-test-spin",
            feature = "lifecycle-test-both-spin"
        ));
    let spin_b = matches!(mode, RunMode::Finish) && cfg!(feature = "lifecycle-test-both-spin");
    let entries = [
        if spin_a {
            user_spin as *const () as u64
        } else {
            match mode {
                RunMode::Finish => lifecycle_program_a as *const () as u64,
                RunMode::Fault => lifecycle_fault as *const () as u64,
                RunMode::Messages => ipc_program_a as *const () as u64,
                RunMode::WaitMessages => wait_program_a as *const () as u64,
                RunMode::RecordCount(_) => record_client as *const () as u64,
            }
        },
        if spin_b {
            user_spin as *const () as u64
        } else if matches!(mode, RunMode::RecordCount(_)) {
            if cfg!(feature = "service-test-worker-fault") {
                lifecycle_fault as *const () as u64
            } else {
                record_worker as *const () as u64
            }
        } else if matches!(mode, RunMode::WaitMessages) {
            if cfg!(feature = "wait-test-peer-fault") {
                lifecycle_fault as *const () as u64
            } else if cfg!(feature = "wait-test-peer-spin") {
                user_spin as *const () as u64
            } else {
                wait_program_b as *const () as u64
            }
        } else if matches!(mode, RunMode::Messages) {
            if cfg!(feature = "ipc-test-peer-fault") {
                lifecycle_fault as *const () as u64
            } else {
                ipc_program_b as *const () as u64
            }
        } else {
            lifecycle_program_b as *const () as u64
        },
    ];
    let mut records: [Record; 2] = core::array::from_fn(|id| {
        let stack = (bottom + id as u64 * 4096, bottom + (id as u64 + 1) * 4096);
        Record {
            frame: Frame {
                elr: entries[id],
                spsr: 0x340,
                user_sp: stack.1,
                ..Frame::default()
            },
            stack,
            lifecycle: Lifecycle::new(),
            reason: "none",
            code: 0,
            value: 0,
            calls: 0,
        }
    });
    if let RunMode::RecordCount(record) = mode {
        records[0].frame.registers[0] = record.raw();
    }
    let ipc_enabled = matches!(
        mode,
        RunMode::Messages | RunMode::WaitMessages | RunMode::RecordCount(_)
    );
    let wait_enabled = matches!(mode, RunMode::WaitMessages | RunMode::RecordCount(_));
    let mut trace = [0; MAX_EVENTS];
    let mut length = 0;
    // SAFETY: sole owner of the mapped user region, one masked EL1 CPU, no DMA or
    // active invocation. Both half-stacks are zeroed before initial user entry.
    unsafe {
        ptr::write_bytes(bottom as *mut u8, 0, 8192);
    }
    let mut cursor = 0;
    let mut mailboxes = Mailboxes::new([0b10, 0b01]);
    let mut stats = IpcStats::default();
    while let Some(id) = next_ready(records.each_ref().map(|r| r.lifecycle.state), cursor) {
        if length == MAX_EVENTS {
            return Err("schedule_event_overflow");
        }
        let task = &mut records[id];
        if !valid_context(&task.frame, task.stack, text) {
            return Err("schedule_bad_context");
        }
        task.lifecycle.start()?;
        // SAFETY: valid_context checks EL0 mode/masks, executable user PC and this
        // task's assigned aligned stack. Stored frames are kernel-only; initial
        // GPRs are zero except the validated scalar service ID in client x0. The input and interrupt Observation are separate objects.
        let observation = unsafe { run_turn(timer, mmu, TaskSpace::from_index(id)?, &task.frame) }?;
        task.frame = observation.frame;
        let mut ipc_event = None;
        let event = if !valid_context(&task.frame, task.stack, text) {
            task.reason = "bad_context";
            Event::Fault
        } else if observation.vector == 9 {
            Event::Preempt
        } else if observation.vector == 8 && task.frame.esr == 0x5600_0000 {
            if task.frame.registers[8] == 2 && task.frame.registers[0] <= 255 {
                task.code = task.frame.registers[0];
                task.value = task.frame.registers[1];
                Event::Exit
            } else if ipc_enabled
                && (matches!(task.frame.registers[8], 10 | 11)
                    || (wait_enabled && task.frame.registers[8] == 12))
            {
                task.calls += 1;
                stats.calls[id] = task.calls;
                if task.calls == MAX_CALLS {
                    // The sixteenth request terminates without queue side effects.
                    Event::CallLimit
                } else {
                    let caller =
                        TaskId::from_raw(id as u64 + 1).map_err(|_| "invalid_task_space")?;
                    let regs = &mut task.frame.registers;
                    let send = regs[8] == 10;
                    let wait = regs[8] == 12;
                    let result = if send {
                        mailboxes
                            .send(caller, regs[0], regs[1], regs[2])
                            .map(|()| None)
                    } else if wait {
                        mailboxes.receive_wait(caller)
                    } else {
                        mailboxes.receive(caller).map(Some)
                    };
                    let pending = wait && matches!(result, Ok(None));
                    regs[..4].fill(0);
                    match result {
                        Ok(Some(message)) => {
                            regs[1] = message.sender.raw();
                            regs[2] = u64::from(message.kind);
                            regs[3] = message.value;
                            stats.received += 1;
                        }
                        Ok(None) if send => stats.sent += 1,
                        Ok(None) => stats.blocked += 1,
                        Err(error) => {
                            if error == crate::ipc::Error::Empty {
                                stats.empty += 1;
                            } else {
                                stats.rejected += 1;
                            }
                            regs[0] = error as u64;
                        }
                    }
                    ipc_event = Some(if send { b'S' } else { b'R' });
                    if pending { Event::Wait } else { Event::Yield }
                }
            } else {
                task.reason = "bad_syscall";
                Event::Fault
            }
        } else {
            task.reason = "exception";
            Event::Fault
        };
        task.lifecycle.finish(event)?;
        let code = match task.lifecycle.state {
            State::Ready => ipc_event.unwrap_or(b'P'),
            State::Exited => b'E',
            State::Faulted => b'F',
            State::LimitReached => b'L',
            State::Waiting => b'D',
            State::Running | State::Deadlocked => return Err("schedule_bad_transition"),
        };
        if task.lifecycle.state.is_terminal() {
            mailboxes.close(TaskId::from_raw(id as u64 + 1).map_err(|_| "invalid_task_space")?);
        }
        record_event(&mut trace, &mut length, id, code)?;
        // Kernel is masked on one CPU: empty-check -> Waiting -> send/close wake
        // cannot race. No EL0 turn happens inside this transition sequence.
        settle_waiters(
            &mut records,
            &mut mailboxes,
            &mut stats,
            &mut trace,
            &mut length,
        )?;
        cursor = id ^ 1;
    }
    // This diagnostic has no external sender or device wake source. All remaining
    // waiters are mutually stuck; end the batch explicitly instead of spinning.
    for (id, task) in records.iter_mut().enumerate() {
        if task.lifecycle.state == State::Waiting {
            task.lifecycle.deadlock()?;
            mailboxes.close(TaskId::from_raw(id as u64 + 1).map_err(|_| "invalid_task_space")?);
            stats.deadlocks += 1;
            record_event(&mut trace, &mut length, id, b'X')?;
        }
    }
    Ok(RunReport {
        records: records.map(|task| TaskResult {
            lifecycle: task.lifecycle,
            reason: task.reason,
            code: task.code,
            value: task.value,
            fault: FaultInfo {
                esr: task.frame.esr,
                elr: task.frame.elr,
                far: task.frame.far,
                spsr: task.frame.spsr,
                user_sp: task.frame.user_sp,
            },
        }),
        trace,
        length,
        ipc: ipc_enabled.then_some(stats),
        waiting: wait_enabled,
    })
}
fn valid_context(frame: &Frame, stack: (u64, u64), text: (u64, u64)) -> bool {
    frame.spsr & !0xf000_0000 == 0x340
        && frame.elr >= text.0
        && frame.elr < text.1
        && frame.elr.is_multiple_of(4)
        && frame.user_sp >= stack.0
        && frame.user_sp <= stack.1
        && frame.user_sp.is_multiple_of(16)
}

fn record_event(
    trace: &mut [u8; MAX_EVENTS],
    length: &mut usize,
    id: usize,
    event: u8,
) -> Result<(), &'static str> {
    if *length == MAX_EVENTS {
        return Err("schedule_event_overflow");
    }
    trace[*length] = ((id as u8) << 7) | event;
    *length += 1;
    Ok(())
}
fn settle_waiters(
    records: &mut [Record; 2],
    mailboxes: &mut Mailboxes,
    stats: &mut IpcStats,
    trace: &mut [u8; MAX_EVENTS],
    length: &mut usize,
) -> Result<(), &'static str> {
    for (id, task) in records.iter_mut().enumerate() {
        if task.lifecycle.state != State::Waiting {
            continue;
        }
        let caller = TaskId::from_raw(id as u64 + 1).map_err(|_| "invalid_task_space")?;
        let result = mailboxes.receive_wait(caller);
        let code = match result {
            Ok(None) => continue,
            Ok(Some(message)) => {
                task.frame.registers[..4].copy_from_slice(&[
                    0,
                    message.sender.raw(),
                    u64::from(message.kind),
                    message.value,
                ]);
                stats.received += 1;
                stats.woken += 1;
                b'W'
            }
            Err(crate::ipc::Error::Closed) => {
                task.frame.registers[..4].copy_from_slice(&[5, 0, 0, 0]);
                stats.rejected += 1;
                stats.closed += 1;
                b'C'
            }
            Err(_) => return Err("ipc_wait_invariant"),
        };
        task.lifecycle.wake()?;
        record_event(trace, length, id, code)?;
    }
    Ok(())
}

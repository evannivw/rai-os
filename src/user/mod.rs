//! Single synchronous EL0 invocation; no loader, scheduler or multi-process runtime.
mod faults;
mod policy;
mod program;
use crate::context::Frame;
use core::{
    arch::global_asm,
    fmt, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};
use policy::*;
global_asm!(include_str!("transition.S"));

struct Session {
    report: Report,
    ranges: [(u64, u64); 2],
    text: (u64, u64),
}
// Only the single synchronous invocation owns this pointer; never exposed to EL0.
static ACTIVE: AtomicPtr<Session> = AtomicPtr::new(ptr::null_mut());
pub struct Report {
    output: [u8; MAX_OUTPUT],
    length: usize,
    calls: u64,
    rejected: u64,
    code: u64,
    fault: Option<(u64, u64, u64, u64, u64)>,
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((esr, pc, far, spsr, sp)) = self.fault {
            write!(
                f,
                "mode=EL0; state=faulted; esr=0x{esr:016x}; pc=0x{pc:016x}; far_raw=0x{far:016x}; spsr=0x{spsr:016x}; sp=0x{sp:016x}; syscalls={}",
                self.calls
            )
        } else {
            // Only printable ASCII is committed to output by dispatch.
            let text = core::str::from_utf8(&self.output[..self.length]).map_err(|_| fmt::Error)?;
            write!(
                f,
                "mode=EL0; state=exited; code={}; syscalls={}; rejected={}; output={text}",
                self.code, self.calls, self.rejected
            )
        }
    }
}

pub fn run() -> Result<Report, &'static str> {
    unsafe extern "C" {
        static __user_text_start: u8;
        static __user_text_end: u8;
        static __user_rodata_end: u8;
        static mut __user_stack_bottom: [u8; 8192];
        static __user_stack_top: u8;
        fn user_enter();
    }
    let stack_start = ptr::addr_of_mut!(__user_stack_bottom) as u64;
    let stack_end = ptr::addr_of!(__user_stack_top) as u64;
    let mut session = Session {
        report: Report {
            output: [0; MAX_OUTPUT],
            length: 0,
            calls: 0,
            rejected: 0,
            code: 0,
            fault: None,
        },
        ranges: [
            (
                ptr::addr_of!(__user_text_end) as u64,
                ptr::addr_of!(__user_rodata_end) as u64,
            ),
            (stack_start, stack_end),
        ],
        text: (
            ptr::addr_of!(__user_text_start) as u64,
            ptr::addr_of!(__user_text_end) as u64,
        ),
    };
    ACTIVE
        .compare_exchange(
            ptr::null_mut(),
            &mut session,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .map_err(|_| "user_busy")?;
    // SAFETY: linker-owned, mapped user stack, no live user invocation. Clear old
    // contents before entry. Single CPU and masked IRQs; no DMA or competing owner.
    unsafe {
        ptr::write_bytes(
            stack_start as *mut u8,
            0,
            (stack_end - stack_start) as usize,
        );
    }
    // SAFETY: live exclusive Session remains on the suspended kernel stack; only
    // guarded lower-EL dispatch borrows it during this call. Assembly preserves C
    // callee-saved state and returns only through exit/fault termination. The
    // built-in program is bounded; arbitrary untrusted binaries are not accepted.
    unsafe { user_enter() };
    ACTIVE.store(ptr::null_mut(), Ordering::Release);
    Ok(session.report)
}

// Called exclusively from vector 8 with a live saved kernel-stack frame. Returning
// true terminates the invocation via a fixed assembly trampoline, not user input.
pub fn dispatch(frame: &mut Frame) -> bool {
    let active = ACTIVE.load(Ordering::Acquire);
    if active.is_null() {
        crate::exceptions::exception_report(
            8,
            frame.esr,
            frame.elr,
            frame.spsr,
            frame.far,
            frame.user_sp,
        );
    }
    // SAFETY: run holds Session alive and does not access it during user_enter.
    // The assembly guard and masked interrupts exclude recursive/parallel borrows.
    let session = unsafe { &mut *active };
    if frame.esr != 0x5600_0000
        || frame.spsr & !0xf000_0000 != 0x3c0
        || frame.elr < session.text.0
        || frame.elr >= session.text.1
        || frame.user_sp < session.ranges[1].0
        || frame.user_sp > session.ranges[1].1
        || !frame.user_sp.is_multiple_of(16)
    {
        session.report.fault = Some((frame.esr, frame.elr, frame.far, frame.spsr, frame.user_sp));
        return true;
    }
    session.report.calls += 1;
    let result = match frame.registers[8] {
        1 => write(session, frame.registers[0], frame.registers[1]),
        2 if frame.registers[0] <= 255 => {
            session.report.code = frame.registers[0];
            return true;
        }
        2 => Err(BAD_LENGTH),
        _ => Err(UNKNOWN_CALL),
    };
    match result {
        Ok(value) => {
            frame.registers[0] = value;
            frame.registers[1] = 0;
        }
        Err(error) => {
            session.report.rejected += 1;
            frame.registers[0] = 0;
            frame.registers[1] = error;
        }
    }
    false
}

fn write(session: &mut Session, address: u64, length: u64) -> Result<u64, u64> {
    let length = readable(address, length, session.ranges)?;
    if length > MAX_OUTPUT - session.report.length {
        return Err(OUTPUT_FULL);
    }
    let mut copy = [0u8; MAX_OUTPUT];
    for (index, byte) in copy[..length].iter_mut().enumerate() {
        // SAFETY: the entire span was checked against readable user-only mappings.
        // No concurrent user execution/remapping/DMA. Copy before validating text;
        // no reference to user memory escapes, and rejected bytes are not committed.
        *byte = unsafe { ptr::read_volatile((address as usize + index) as *const u8) };
    }
    if !copy[..length]
        .iter()
        .all(|b| b.is_ascii_graphic() || *b == b' ')
    {
        return Err(BAD_TEXT);
    }
    let start = session.report.length;
    session.report.output[start..start + length].copy_from_slice(&copy[..length]);
    session.report.length += length;
    Ok(length as u64)
}

//! Two-context diagnostics and a bounded task lifecycle on one CPU.
mod ipc_programs;
mod isolation;
mod lifecycle;
pub use isolation::{IsolationReport, check as isolate};
mod programs;
mod runtime;
use crate::{
    clock,
    context::Frame,
    mmu::{Mmu, TaskSpace},
    timer::{SLICE_MS, Timer},
};
use core::{
    arch::global_asm,
    fmt, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};
pub use runtime::{RunMode, RunReport, run};
global_asm!(include_str!("resume.S"));
global_asm!(include_str!("programs.S"),
    corrupt = const cfg!(feature = "schedule-test-corrupt-register") as u8,
    fault = const cfg!(feature = "schedule-test-user-fault") as u8);
const TURNS: usize = 12;
struct Observation {
    frame: Frame,
    vector: u64,
    at: u64,
    elapsed_ms: u64,
}
static ACTIVE: AtomicPtr<Observation> = AtomicPtr::new(ptr::null_mut());
pub struct Report {
    trace: [u8; TURNS],
    progress: [u64; TURNS],
    elapsed: [u64; 2],
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let trace = core::str::from_utf8(&self.trace).map_err(|_| fmt::Error)?;
        write!(
            f,
            "tasks=2; slice_ms={SLICE_MS}; irqs=12; switches=11; trace={trace}; a_turns=6; b_turns=6; a_ms={}; b_ms={}; progress=",
            self.elapsed[0], self.elapsed[1]
        )?;
        for (index, value) in self.progress.iter().enumerate() {
            if index != 0 {
                f.write_str(",")?;
            }
            write!(f, "{value}")?;
        }
        f.write_str("; context=true; stacks=true")
    }
}
pub(crate) fn is_active() -> bool {
    !ACTIVE.load(Ordering::Acquire).is_null()
}

/// Save a paused EL0 frame into kernel RAM. The caller already handles GIC EOI.
/// A synchronous event also yields, but check rejects it rather than resuming it.
pub(crate) fn capture(frame: &Frame, vector: u64) -> bool {
    let active = ACTIVE.load(Ordering::Acquire);
    if active.is_null() {
        return false;
    }
    // SAFETY: check keeps Observation alive while its caller is suspended. All
    // handlers run masked on one CPU with the shared non-nesting guard set.
    // The caller does not read/write Observation until the assembly call returns.
    let observation = unsafe { &mut *active };
    observation.frame = *frame;
    observation.vector = vector;
    observation.at = clock::counter();
    true
}

pub fn check(timer: &mut Timer, mmu: &Mmu) -> Result<Report, &'static str> {
    if is_active() || crate::user::is_active() {
        return Err("schedule_busy");
    }
    unsafe extern "C" {
        static mut __user_stack_bottom: [u8; 8192];
        fn schedule_program_a();
        fn schedule_program_b();
        static schedule_program_a_end: u8;
        static schedule_program_b_end: u8;
    }
    let bottom = ptr::addr_of_mut!(__user_stack_bottom) as u64;
    let entries = [
        schedule_program_a as *const () as u64,
        schedule_program_b as *const () as u64,
    ];
    let ends = [
        ptr::addr_of!(schedule_program_a_end) as u64,
        ptr::addr_of!(schedule_program_b_end) as u64,
    ];
    // Each context owns one 4 KiB half of the existing 8 KiB user region for the
    // diagnostic lifetime. A task map permits only its own page at EL0.
    let stacks = [bottom + 4096 - 16, bottom + 8192 - 16];
    let flags = [0xa000_0340, 0x5000_0340];
    let seeds = [0xa5a5_0000_0000_1000_u64, 0x5a5a_0000_0000_2000];
    let mut contexts: [Frame; 2] = core::array::from_fn(|id| {
        let mut frame = Frame {
            elr: entries[id],
            spsr: flags[id],
            user_sp: stacks[id],
            ..Frame::default()
        };
        for i in 1..31 {
            frame.registers[i] = seeds[id] + i as u64;
        }
        frame
    });
    // SAFETY: exclusive linker-owned user RAM, no user invocation, single masked
    // kernel CPU, no DMA. No pointers/references to these bytes escape to Rust.
    unsafe {
        ptr::write_bytes(bottom as *mut u8, 0, 8192);
        for id in 0..2 {
            ptr::write_volatile((stacks[id] + 8) as *mut u64, seeds[id]);
        }
    }
    let mut report = Report {
        trace: [0; TURNS],
        progress: [0; TURNS],
        elapsed: [0; 2],
    };
    for turn in 0..TURNS {
        let id = turn % 2;
        let input = contexts[id]; // Read-only input separate from IRQ output.
        // SAFETY: initial frames use fixed user entries/stacks and EL0 status;
        // subsequent frames passed the validation below before being retained.
        let observation = unsafe { run_turn(timer, mmu, TaskSpace::from_index(id)?, &input) }?;
        if observation.vector != 9 {
            return Err("schedule_user_fault");
        }
        let saved = observation.frame;
        let step = if id == 0 { 1 } else { 3 };
        // SAFETY: EL0 is paused; both aligned words are inside its assigned half.
        // Volatile loads observe actual user stores. No remapping/concurrent writer.
        let (stored, marker) = unsafe {
            (
                ptr::read_volatile(stacks[id] as *const u64),
                ptr::read_volatile((stacks[id] + 8) as *const u64),
            )
        };
        if saved.elr < entries[id]
            || saved.elr >= ends[id]
            || !saved.elr.is_multiple_of(4)
            || saved.spsr != flags[id]
            || saved.user_sp != stacks[id]
            || saved.registers[1..] != input.registers[1..]
            || saved.registers[0] <= input.registers[0]
            || !saved.registers[0].is_multiple_of(step)
            || (stored != saved.registers[0]
                && stored.checked_add(step) != Some(saved.registers[0]))
            || marker != seeds[id]
        {
            return Err("schedule_context_corrupt");
        }
        contexts[id] = saved; // Keep the interrupted PC and registers for its next turn.
        report.trace[turn] = b'A' + id as u8;
        report.progress[turn] = saved.registers[0];
        report.elapsed[id] += observation.elapsed_ms;
    }
    // Both contexts remain paused; this bounded diagnostic discards them here.
    Ok(report)
}

/// # Safety
/// Input is kernel-owned and initialized, with a validated executable EL0 PC,
/// assigned aligned user SP and EL0t/IRQ-enabled PSTATE. One masked kernel CPU,
/// installed vectors, exclusive user RAM, no active user/schedule invocation.
/// The input is read-only and separate from the handler's output Observation.
unsafe fn run_turn(
    timer: &mut Timer,
    mmu: &Mmu,
    space: TaskSpace,
    input: &Frame,
) -> Result<Observation, &'static str> {
    // SAFETY: validated user frame, one masked kernel CPU; inner returns masked
    // after exception capture/cleanup. Base map restoration also covers errors.
    unsafe { mmu.with_task(space, || run_mapped_turn(timer, input)) }?
}

/// # Safety
/// Same frame/entry contract as run_turn; the selected task map is already active.
unsafe fn run_mapped_turn(timer: &mut Timer, input: &Frame) -> Result<Observation, &'static str> {
    unsafe extern "C" {
        fn schedule_enter(input: *const Frame);
    }
    let budget = timer.start_slice()?;
    let mut observation = Observation {
        frame: Frame::default(),
        vector: 0,
        at: 0,
        elapsed_ms: 0,
    };
    ACTIVE.store(&mut observation, Ordering::Release);
    // SAFETY: caller establishes the validated frame contract above. Assembly
    // retains the C caller and returns masked via the fixed user_return path.
    unsafe { schedule_enter(input) };
    ACTIVE.store(ptr::null_mut(), Ordering::Release);
    timer.finish_user_budget()?;
    if observation.vector == 9 && !budget.expired(observation.at) {
        return Err("schedule_timer_early");
    }
    observation.elapsed_ms = budget.elapsed_ms(observation.at);
    Ok(observation)
}

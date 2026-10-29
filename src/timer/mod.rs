//! Controlled EL1 timer IRQ diagnostic; normal kernel/EL0 execution stays masked.
mod hardware;
use crate::{
    clock,
    context::{Frame, Snapshot},
};
use core::{
    arch::global_asm,
    fmt, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};
global_asm!(include_str!("probe.S"));

pub struct Timer {
    period: u32,
    frequency: u32,
}
struct Round {
    period: u32,
    ticks: u64,
    first: u64,
    last: u64,
}
static ACTIVE: AtomicPtr<Round> = AtomicPtr::new(ptr::null_mut());
pub struct Report {
    elapsed_ms: u64,
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "irq=30; ticks=6; rounds=2; registers=31; flags=true; stack=true; irq_masked=true; timer_disabled=true; elapsed_ms={}",
            self.elapsed_ms
        )
    }
}
impl Timer {
    /// # Safety
    /// Construct once after MMU setup, EL1h, one CPU, all DAIF masked. This owner
    /// has exclusive GICv2/CNTP access; no other driver enables interrupts or DMA.
    pub unsafe fn initialize(frequency: u32) -> Self {
        assert_ne!(frequency, 0);
        hardware::initialize();
        // Approximately 10 ms, rounded up to at least one physical counter tick.
        Self {
            period: u64::from(frequency).div_ceil(100) as u32,
            frequency,
        }
    }
    pub fn check(&mut self) -> Result<Report, &'static str> {
        if hardware::masks() != 0x3c0 {
            return Err("timer_requires_masked_irqs");
        }
        unsafe extern "C" {
            fn timer_probe(output: *mut Snapshot, flags: u64);
        }
        let mut elapsed = 0_u64;
        for flags in [0xa000_0000, 0x5000_0000] {
            let mut snapshot = Snapshot::default();
            let mut round = Round {
                period: self.period,
                ticks: 0,
                first: 0,
                last: 0,
            };
            ACTIVE
                .compare_exchange(
                    ptr::null_mut(),
                    &mut round,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .map_err(|_| "timer_busy")?;
            hardware::start();
            let start = clock::counter();
            hardware::arm(self.period);
            #[cfg(feature = "timer-test-unexpected-irq")]
            hardware::inject_unexpected();
            // SAFETY: aligned initialized snapshot and installed vector 5; the
            // only unmasked window is assembly, whose GPR/flags remain unchanged.
            // IRQ dispatch exclusively borrows round while this caller is paused.
            // The third IRQ redirects only this known loop to its continuation,
            // restoring I=1 before returning to Rust. No nested IRQs or unwinding.
            unsafe { timer_probe(&mut snapshot, flags) };
            hardware::stop();
            ACTIVE.store(ptr::null_mut(), Ordering::Release);
            if !snapshot.matches(flags)
                || round.ticks != 3
                || round.last.saturating_sub(round.first) < u64::from(self.period) * 2
                || hardware::masks() != 0x3c0
                || !hardware::quiescent()
            {
                return Err("timer_check_failed");
            }
            elapsed = elapsed.saturating_add(round.last.saturating_sub(start));
        }
        Ok(Report {
            elapsed_ms: (u128::from(elapsed) * 1000 / u128::from(self.frequency))
                .min(u128::from(u64::MAX)) as u64,
        })
    }
}

pub fn interrupt(frame: &mut Frame) -> Result<(), u32> {
    let acknowledgement = hardware::acknowledge();
    let id = acknowledgement & 1023;
    // Spurious IDs are not active and must not receive EOI or count as a tick.
    if id >= 1020 {
        return Ok(());
    }
    let active = ACTIVE.load(Ordering::Acquire);
    unsafe extern "C" {
        static timer_probe_loop: u8;
        static timer_probe_resume: u8;
    }
    if id != hardware::TIMER_IRQ
        || active.is_null()
        || frame.elr != ptr::addr_of!(timer_probe_loop) as u64
        || frame.spsr & !0xf000_0000 != 0x345
    {
        hardware::disable_timer();
        hardware::complete(acknowledgement);
        return Err(id);
    }
    // SAFETY: check keeps Round alive on its suspended stack; only this handler
    // accesses it during the probe. I=1 and the common entry guard exclude nesting.
    let round = unsafe { &mut *active };
    let now = clock::counter();
    round.ticks += 1;
    if round.ticks == 1 {
        round.first = now;
    }
    round.last = now;
    if round.ticks < 3 {
        hardware::arm(round.period);
    } else {
        hardware::disable_timer();
        frame.elr = ptr::addr_of!(timer_probe_resume) as u64;
        frame.spsr |= 1 << 7; // Return with IRQ masked before snapshot/Rust code.
    }
    hardware::complete(acknowledgement);
    Ok(())
}

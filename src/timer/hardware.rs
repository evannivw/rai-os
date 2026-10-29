//! Sole GICv2/physical-timer adapter for the fixed virt-9.2 board.
use core::{arch::asm, ptr};
const DISTRIBUTOR: usize = 0x0800_0000;
const CPU: usize = 0x0801_0000;
pub const TIMER_IRQ: u32 = 30;
const TIMER_BIT: u32 = 1 << TIMER_IRQ;
fn read(base: usize, offset: usize) -> u32 {
    // SAFETY: private callers use aligned 32-bit registers within the two mapped
    // Device-nGnRnE pages. Single CPU, sole owner, no references to device memory.
    unsafe { ptr::read_volatile((base + offset) as *const u32) }
}
fn write(base: usize, offset: usize, value: u32) {
    // SAFETY: same fixed mapping/alignment/ownership contract as read.
    unsafe { ptr::write_volatile((base + offset) as *mut u32, value) };
}
fn synchronize() {
    // SAFETY: EL1 ordering barrier; ensure device writes reach the GIC before
    // unmask/return/EOI. No Rust memory access is hidden with a nomem annotation.
    unsafe { asm!("dsb sy", "isb", options(nostack, preserves_flags)) };
}
pub fn masks() -> u64 {
    let value;
    // SAFETY: read-only privileged register, valid under the EL1 runtime contract.
    unsafe {
        asm!("mrs {value}, daif", value = out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}
pub fn disable_timer() {
    // SAFETY: EL1 owns CNTP; disable its level signal before GIC deactivation.
    unsafe {
        asm!(
            "msr cntp_ctl_el0, xzr",
            "isb",
            options(nostack, preserves_flags)
        );
    }
}
pub fn arm(period: u32) {
    // SAFETY: positive signed-32-bit interval, EL1 owns CNTP, current count plus
    // interval is the next deadline. Both control and comparator are synchronized.
    unsafe {
        asm!("msr cntp_tval_el0, {period}", "mov x9, #1", "msr cntp_ctl_el0, x9", "isb",
        period = in(reg) u64::from(period), out("x9") _, options(nostack, preserves_flags));
    }
}
pub fn initialize() {
    assert_eq!(masks(), 0x3c0, "timer init needs masked interrupts");
    // SAFETY: sole EL1 timer owner. Deny EL0 timer/counter access and event stream;
    // do not rely on reset values. SCTLR.UMA must also deny user DAIF writes.
    let (access, control): (u64, u64);
    unsafe {
        asm!("msr cntkctl_el1, xzr", "isb", "mrs {access}, cntkctl_el1",
            "mrs {control}, sctlr_el1", access = out(reg) access, control = out(reg) control,
            options(nostack, preserves_flags));
    }
    assert_eq!(access & 0x3ff, 0);
    assert_eq!(control & (1 << 9), 0, "EL0 must not mask IRQs");
    disable_timer();
    write(CPU, 0, 0);
    write(DISTRIBUTOR, 0, 0);
    assert_eq!((read(DISTRIBUTOR, 0xfe8) >> 4) & 15, 2, "GICv2 required");
    // Reset configurable enables/pending/active state; this is the sole IRQ owner.
    let banks = (read(DISTRIBUTOR, 4) & 31) + 1;
    for bank in 0..banks as usize {
        write(DISTRIBUTOR, 0x180 + bank * 4, u32::MAX);
        write(DISTRIBUTOR, 0x280 + bank * 4, u32::MAX);
        write(DISTRIBUTOR, 0x380 + bank * 4, u32::MAX);
    }
    write(DISTRIBUTOR, 0x80, 0); // Group 0; security extensions are disabled.
    write(DISTRIBUTOR, 0xc04, 0); // PPIs level-sensitive, including CNTP.
    write(DISTRIBUTOR, 0x41c, 0x8080_8080); // Priorities for IDs 28..31.
    write(CPU, 4, 0xff); // Priority mask admits priority 0x80.
    write(CPU, 8, 0); // Binary point; no nested IRQs in this increment.
    synchronize();
}
pub fn start() {
    write(DISTRIBUTOR, 0x280, TIMER_BIT);
    write(DISTRIBUTOR, 0x100, TIMER_BIT);
    write(DISTRIBUTOR, 0, 1);
    write(CPU, 0, 1); // Enable Group 0 as IRQ (FIQEn and split EOI are clear).
    synchronize();
}
pub fn stop() {
    disable_timer();
    write(DISTRIBUTOR, 0x180, TIMER_BIT);
    write(DISTRIBUTOR, 0x280, TIMER_BIT);
    write(CPU, 0, 0);
    write(DISTRIBUTOR, 0, 0);
    synchronize();
}
pub fn quiescent() -> bool {
    let control: u64;
    // SAFETY: EL1 read of our timer's enable/mask flags; ISTATUS is ignored.
    unsafe {
        asm!("mrs {value}, cntp_ctl_el0", value = out(reg) control, options(nostack, preserves_flags));
    }
    control & 3 == 0
        && read(CPU, 0) & 3 == 0
        && read(DISTRIBUTOR, 0) & 3 == 0
        && read(DISTRIBUTOR, 0x100) & TIMER_BIT == 0
        && read(DISTRIBUTOR, 0x300) & TIMER_BIT == 0
}
pub fn acknowledge() -> u32 {
    read(CPU, 0xc)
}
pub fn complete(acknowledgement: u32) {
    // Timer must be rearmed/disabled first. Return the complete IAR value.
    synchronize();
    write(CPU, 0x10, acknowledgement);
    synchronize();
}
#[cfg(feature = "timer-test-unexpected-irq")]
pub fn inject_unexpected() {
    write(DISTRIBUTOR, 0xf00, (2 << 24) | 1); // SGI 1, target this CPU.
    synchronize();
}

pub fn arm_deadline(deadline: u64) {
    // SAFETY: EL1 owns CNTP. Absolute compare value is fixed for one invocation;
    // syscalls never reset it. Counter wrap/reset is excluded by the boot contract.
    unsafe {
        asm!("msr cntp_cval_el0, {deadline}", "mov x9, #1", "msr cntp_ctl_el0, x9", "isb",
            deadline = in(reg) deadline, out("x9") _, options(nostack, preserves_flags));
    }
}

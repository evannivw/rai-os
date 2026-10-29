//! Hardware permission diagnostic. No user-supplied addresses or executable code.
use super::{is_active, run_turn};
use crate::{
    context::Frame,
    mmu::{Mmu, TaskSpace},
    timer::Timer,
};
use core::{arch::global_asm, fmt, ptr};

global_asm!(include_str!("isolation.S"));
const ROUNDS: usize = 2;
const MARKERS: [u64; 2] = [0x1111_1111_1111_1111, 0x2222_2222_2222_2222];
#[derive(Clone, Copy, Default)]
struct Fault {
    esr: u64,
    pc: u64,
    far: u64,
}
pub struct IsolationReport {
    faults: [Fault; 4],
    reads: usize,
    writes: usize,
    own: usize,
}
impl fmt::Display for IsolationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rounds={ROUNDS}; reads_denied={}; writes_denied={}; own_accesses={}; memory_unchanged=true; base_restored=true",
            self.reads, self.writes, self.own
        )?;
        for (name, fault) in ["ar", "aw", "br", "bw"].iter().zip(&self.faults) {
            write!(
                f,
                "; {name}_esr=0x{:016x}; {name}_pc=0x{:016x}; {name}_far=0x{:016x}",
                fault.esr, fault.pc, fault.far
            )?;
        }
        Ok(())
    }
}

pub fn check(timer: &mut Timer, mmu: &Mmu) -> Result<IsolationReport, &'static str> {
    if is_active() || crate::user::is_active() {
        return Err("schedule_busy");
    }
    unsafe extern "C" {
        static mut __user_stack_bottom: [u8; 8192];
        fn isolation_own();
        fn isolation_read();
        fn isolation_write();
        static isolation_own_end: u8;
        static isolation_read_end: u8;
        static isolation_write_end: u8;
        static isolation_read_fault: u8;
        static isolation_write_fault: u8;
    }
    let bottom = ptr::addr_of_mut!(__user_stack_bottom) as u64;
    let entries = [
        isolation_own as *const () as u64,
        isolation_read as *const () as u64,
        isolation_write as *const () as u64,
    ];
    let ends = [
        ptr::addr_of!(isolation_own_end) as u64,
        ptr::addr_of!(isolation_read_end) as u64,
        ptr::addr_of!(isolation_write_end) as u64,
    ];
    let fault_pcs = [
        ptr::addr_of!(isolation_read_fault) as u64,
        ptr::addr_of!(isolation_write_fault) as u64,
    ];
    let before = mmu.snapshot();
    let mut report = IsolationReport {
        faults: [Fault::default(); 4],
        reads: 0,
        writes: 0,
        own: 0,
    };
    for _ in 0..ROUNDS {
        // SAFETY: exclusive user RAM, both tasks stopped, base map, masked EL1.
        // Probes use no Rust stack; these patterns cover both whole stack pages.
        unsafe {
            ptr::write_bytes(bottom as *mut u8, 0x11, 4096);
            ptr::write_bytes((bottom + 4096) as *mut u8, 0x22, 4096);
        }
        // Warm both translations with successful EL0 accesses before denying them
        // to the peer. Repeated A/B switches exercise stale-TLB permission risks.
        for id in 0..2 {
            own(timer, mmu, id, bottom, (entries[0], ends[0]))?;
            report.own += 1;
        }
        for id in 0..2 {
            for access in 0..2 {
                let target = bottom + (id ^ 1) as u64 * 4096;
                let (vector, frame) = probe(
                    timer,
                    mmu,
                    id,
                    bottom,
                    (entries[access + 1], ends[access + 1]),
                    target,
                    0,
                )?;
                let expected = if access == 0 {
                    0x9200_000f
                } else {
                    0x9200_004f
                };
                if vector != 8
                    || frame.esr != expected
                    || frame.elr != fault_pcs[access]
                    || frame.far != target
                {
                    return Err("isolation_expected_fault");
                }
                report.faults[id * 2 + access] = Fault {
                    esr: frame.esr,
                    pc: frame.elr,
                    far: frame.far,
                };
                if access == 0 {
                    report.reads += 1;
                } else {
                    report.writes += 1;
                }
                // SAFETY: paused EL0, both pages are kernel-readable ordinary RAM;
                // volatile reads observe every byte after the attempted peer write.
                for offset in 0..8192 {
                    let expected = if offset < 4096 { 0x11 } else { 0x22 };
                    if unsafe { ptr::read_volatile((bottom + offset) as *const u8) } != expected {
                        return Err("isolation_memory_changed");
                    }
                }
                // Check integrity BEFORE the own-page probe writes its marker,
                // so that the positive control cannot repair/hide a changed byte.
                own(timer, mmu, id ^ 1, bottom, (entries[0], ends[0]))?;
                report.own += 1;
            }
        }
    }
    if mmu.snapshot() != before {
        return Err("mmu_restore_state");
    }
    Ok(report)
}
fn own(
    timer: &mut Timer,
    mmu: &Mmu,
    id: usize,
    bottom: u64,
    code: (u64, u64),
) -> Result<(), &'static str> {
    let (vector, frame) = probe(
        timer,
        mmu,
        id,
        bottom,
        code,
        bottom + id as u64 * 4096,
        MARKERS[id],
    )?;
    if vector != 8
        || frame.esr != 0x5600_0000
        || frame.registers[8] != 2
        || frame.registers[0] != 0
        || frame.registers[1] != MARKERS[id]
    {
        return Err("isolation_own_access_failed");
    }
    Ok(())
}
/// Fixed linked probes may be preempted before their access; resume only a checked
/// frame in the SAME task map. A sixth preemption aborts this bounded diagnostic.
fn probe(
    timer: &mut Timer,
    mmu: &Mmu,
    id: usize,
    bottom: u64,
    code: (u64, u64),
    address: u64,
    value: u64,
) -> Result<(u64, Frame), &'static str> {
    let space = TaskSpace::from_index(id)?;
    let sp = bottom + (id as u64 + 1) * 4096;
    let mut frame = Frame {
        elr: code.0,
        user_sp: sp,
        spsr: 0x340,
        ..Frame::default()
    };
    frame.registers[0] = address;
    frame.registers[1] = value;
    for _ in 0..6 {
        // SAFETY: fixed user code, aligned own stack, validated EL0 status, no
        // kernel register leaks; one masked CPU and exclusive invocation ownership.
        let observed = unsafe { run_turn(timer, mmu, space, &frame) }?;
        frame = observed.frame;
        if frame.spsr != 0x340
            || frame.user_sp != sp
            || frame.elr < code.0
            || frame.elr >= code.1
            || !frame.elr.is_multiple_of(4)
        {
            return Err("isolation_bad_context");
        }
        if observed.vector != 9 {
            return Ok((observed.vector, frame));
        }
    }
    Err("isolation_probe_limit")
}

//! Bounded EL1 context diagnostic. No userspace, interrupts or task switching.
pub mod policy;
use core::{
    arch::global_asm,
    mem::{offset_of, size_of},
};
global_asm!(include_str!("probe.S"));

// Layout is shared with synchronous_entry in exceptions.S. All bytes initialized.
#[repr(C, align(16))]
pub struct Frame {
    registers: [u64; 31],
    pub elr: u64,
    pub spsr: u64,
    pub esr: u64,
    pub far: u64,
    reserved: u64,
}
const _: () = {
    assert!(size_of::<Frame>() == 288);
    assert!(offset_of!(Frame, elr) == 248);
    assert!(offset_of!(Frame, spsr) == 256);
    assert!(offset_of!(Frame, esr) == 264);
    assert!(offset_of!(Frame, far) == 272);
};

#[repr(C, align(16))]
#[derive(Default)]
struct Snapshot {
    registers: [u64; 31],
    nzcv: u64,
    sp: u64,
    daif: u64,
    before_sp: u64,
    reserved: u64,
}
const _: () = {
    assert!(size_of::<Snapshot>() == 288);
    assert!(offset_of!(Snapshot, nzcv) == 248);
    assert!(offset_of!(Snapshot, sp) == 256);
    assert!(offset_of!(Snapshot, daif) == 264);
    assert!(offset_of!(Snapshot, before_sp) == 272);
};

pub fn check() -> Result<&'static str, &'static str> {
    unsafe extern "C" {
        fn context_probe(output: *mut Snapshot, flags: u64);
    }
    for flags in [0xa000_0000, 0x5000_0000] {
        let mut snapshot = Snapshot::default();
        // SAFETY: fixed kernel runtime is single-CPU EL1h, interrupts masked,
        // valid mapped stack/vectors. Exclusive buffer matches assembly layout.
        // Probe preserves C callee-saved registers and writes only this buffer;
        // the allowlisted SVC has no side effects on interrupted Rust state.
        unsafe { context_probe(&mut snapshot, flags) };
        if snapshot
            .registers
            .iter()
            .enumerate()
            .any(|(i, value)| *value != 0xa5a5_0000_0000_1000 + i as u64)
            || snapshot.nzcv != flags
            || snapshot.daif != 0x3c0
            || snapshot.sp != snapshot.before_sp
            || snapshot.sp == 0
            || !snapshot.sp.is_multiple_of(16)
        {
            return Err("context_corrupt");
        }
    }
    Ok("round_trips=2; registers=31; flags=true; stack=true; resumed=true")
}

#[cfg(any(
    feature = "context-test-unknown-svc",
    feature = "context-test-wrong-site"
))]
mod self_test;
#[cfg(any(
    feature = "context-test-unknown-svc",
    feature = "context-test-wrong-site"
))]
pub use self_test::run as self_test;

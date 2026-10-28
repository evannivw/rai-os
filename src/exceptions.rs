//! EL1 diagnostics plus one allowlisted synchronous return; no general recovery.
use core::arch::{asm, global_asm};

global_asm!(include_str!("exceptions.S"));

// SAFETY: assembly fatal entry or synchronous dispatch calls this after capturing
// EL1 registers, masking interrupts and selecting our dedicated aligned stack.
// This fatal reporter never resumes interrupted code or unwinds.
// One CPU owns PL011; the nested-exception guard prevents recursive reporting.
#[unsafe(no_mangle)]
pub extern "C" fn exception_report(
    vector: u64,
    esr: u64,
    elr: u64,
    spsr: u64,
    far_raw: u64,
    entry_sp: u64,
) -> ! {
    let handler_sp: u64;
    // SAFETY: reading the current SP touches neither memory nor flags; assembly
    // has already selected a valid, aligned emergency stack in mapped RAM.
    unsafe {
        asm!("mov {value}, sp", value = out(reg) handler_sp,
            options(nomem, nostack, preserves_flags));
    }
    // A fault may occur before normal UART setup. This fatal path takes over the
    // device permanently; it must not return to a partially interrupted writer.
    crate::uart::init();
    crate::uart::write_display(format_args!(
        "EXCEPTION: vector={vector}; esr=0x{esr:016x}; ec=0x{:02x}; elr=0x{elr:016x}; spsr=0x{spsr:016x}; far_raw=0x{far_raw:016x}; entry_sp=0x{entry_sp:016x}; handler_sp=0x{handler_sp:016x}\n",
        (esr >> 26) & 0x3f,
    ));
    // ESR/FAR interpretation depends on vector/class; FAR is not valid for BRK.
    crate::uart::write("HALTED: exception is fatal; restart QEMU\n");
    loop {
        // SAFETY: interrupts remain masked. WFE waits without accessing memory;
        // wakeups repeat the loop and cannot resume the faulting instruction.
        unsafe { asm!("wfe", options(nomem, nostack, preserves_flags)) };
    }
}

#[cfg(feature = "exception-self-test")]
global_asm!(
    ".section .text.exception_test, \"ax\"",
    ".global exception_test_trigger",
    ".global exception_test_instruction",
    ".type exception_test_trigger, %function",
    "exception_test_trigger:",
    "exception_test_instruction:",
    "brk #0x5241",
    "ret",
);

#[cfg(feature = "exception-self-test")]
pub fn self_test() {
    unsafe extern "C" {
        fn exception_test_trigger();
    }
    crate::uart::write("SELF-TEST: deliberate BRK #0x5241\n");
    // SAFETY: boot installed EL1 vectors and the diagnostic stack. The assembly
    // function uses the C ABI and deliberately traps; the fatal handler never
    // returns. Test code keeps a RET so an erroneous return is observable as a
    // console banner and fails verification instead of falling into random code.
    unsafe { exception_test_trigger() };
}

// SAFETY: synchronous_entry saves a fully initialized, aligned Frame on the
// live normal stack and passes its exclusive address while using the dedicated
// exception stack. One CPU, DAIF masked, no unwinding and no recursive entry.
// This immutable borrow ends before assembly restores the frame and executes ERET.
#[unsafe(no_mangle)]
pub extern "C" fn exception_dispatch(frame: &crate::context::Frame) {
    unsafe extern "C" {
        static context_probe_resume: u8;
    }
    let expected_return = core::ptr::addr_of!(context_probe_resume) as u64;
    if crate::context::policy::may_resume(frame.esr, frame.elr, frame.spsr, expected_return) {
        return;
    }
    exception_report(
        4,
        frame.esr,
        frame.elr,
        frame.spsr,
        frame.far,
        frame as *const _ as u64 + core::mem::size_of::<crate::context::Frame>() as u64,
    )
}

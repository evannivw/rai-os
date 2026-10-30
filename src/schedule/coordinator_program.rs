//! Persistent EL0 policy: observe a goal, admit a matching proposal, dispatch and remember.
//! The host transports scalar inputs; it does not own this program's goal/result state.
use super::ipc_programs::{call, expect, finish};
#[inline(always)]
fn publish(state: u64, ticket: u64, result: u64, completed: u64, event: u64) {
    let (mut a, mut b, mut c, mut d) = (state, ticket, result, completed);
    // SAFETY: coordinator-only scalar publication ABI; no pointers. Kernel
    // preserves x4 and all other undeclared GPRs; x0-x3 return zero on success.
    unsafe {
        core::arch::asm!("svc #0", in("x8") 14_u64, in("x4") event,
        inout("x0") a, inout("x1") b, inout("x2") c, inout("x3") d, options(nostack));
    }
    expect((a, b, c, d), 0);
}
// SAFETY: fixed EL0 entry, retained private A stack; only scalar IPC/gateway calls.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn guest_coordinator() -> ! {
    let mut state = 0_u64;
    let mut ticket = 0_u64;
    let mut result = 0_u64;
    let mut completed = 0_u64;
    publish(state, ticket, result, completed, 0);
    loop {
        let input = call(13, 0, 0, 0);
        if input.0 != 0 || input.3 == 0 || input.3 > 6 {
            finish(1, 0);
        }
        if cfg!(feature = "coord-test-fault") {
            // SAFETY: intentional forbidden EL0 kernel read; the runtime must contain it.
            unsafe {
                core::arch::asm!("ldr x9, [x10]", in("x10") 0x4008_0000_u64,
                out("x9") _, options(nostack));
            }
        }
        if cfg!(feature = "coord-test-spin") {
            loop {
                core::hint::spin_loop();
            }
        }
        if cfg!(feature = "coord-test-flood") {
            loop {
                let _ = call(10, 0, 0, 0);
            }
        }
        let event;
        if input.1 == 30 {
            if result >> 32 == input.3 {
                event = 2; // Duplicate observation retains ticket, result and completion.
            } else {
                state = 1;
                ticket = input.2;
                result = input.3 << 32;
                event = 1; // Await a proposal for this exact goal ticket.
            }
        } else if input.1 == 31 {
            if state == 0 {
                event = 7;
            } else if input.2 != ticket {
                event = 4;
            } else if input.3 != result >> 32 {
                event = 5;
            } else if state == 2 {
                event = 6;
            } else {
                expect(call(10, 2, 20, input.3), 0);
                let reply = call(12, 0, 0, 0);
                let sequence = call(12, 0, 0, 0);
                if reply.0 != 0
                    || reply.1 != 2
                    || reply.2 != 21
                    || reply.3 >> 32 != input.3
                    || sequence.0 != 0
                    || sequence.1 != 2
                    || sequence.2 != 23
                    || sequence.3 != completed + 1
                {
                    finish(2, 0);
                }
                result = reply.3;
                completed = sequence.3;
                state = 2;
                event = 3;
            }
        } else {
            finish(3, 0);
        }
        if cfg!(feature = "coord-test-forged-completion") {
            // A well-shaped count must not bypass result attestation by calling it a duplicate.
            publish(2, input.2, (2_u64 << 32) | (30 << 16) | 7, completed, 2);
            let _ = call(13, 0, 0, 0);
            finish(4, 0);
        }
        let reported_ticket = if cfg!(feature = "coord-test-bad-report") {
            u64::MAX
        } else {
            ticket
        };
        publish(state, reported_ticket, result, completed, event);
    }
}

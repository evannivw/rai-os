//! Fixed EL0 clients of the scalar send/receive ABI. No shared writable memory.
use core::arch::asm;
#[inline(always)]
fn call(number: u64, mut a: u64, mut b: u64, mut c: u64) -> (u64, u64, u64, u64) {
    // Deliberately nonzero: the kernel must not accept this as a sender identity
    // or expose it as receive output after a failed call.
    let mut d = 999_u64;
    // SAFETY: the runtime preserves all GPRs except declared result x0-x3. Inputs
    // are scalars, never dereferenced. SVC may reschedule; own stack/map resumes.
    unsafe {
        asm!("svc #0", in("x8") number, inout("x0") a, inout("x1") b,
        inout("x2") c, inout("x3") d, options(nostack));
    }
    (a, b, c, d)
}
#[inline(always)]
fn finish(code: u64, value: u64) -> ! {
    // SAFETY: exit terminates this task. Fallback remains preemptible at EL0.
    unsafe {
        asm!("svc #0", "2: b 2b", in("x8") 2_u64, in("x0") code,
        in("x1") value, options(noreturn));
    }
}
#[inline(always)]
fn expect(result: (u64, u64, u64, u64), status: u64) {
    if result.0 != status || result.1 != 0 || result.2 != 0 || result.3 != 0 {
        finish(1, result.0);
    }
}
#[inline(always)]
fn receive() -> (u64, u64, u64, u64) {
    loop {
        let result = call(11, 0, 0, 0);
        if result.0 != 4 {
            return result;
        }
        expect(result, 4);
    }
}
// SAFETY: unique linked user entry, only entered by validated ERET at EL0.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn ipc_program_a() -> ! {
    if cfg!(feature = "ipc-test-rejections") {
        expect(call(10, 0, 1, 7), 1);
        expect(call(10, u64::MAX, 1, 7), 1);
        expect(call(10, 1, 1, 7), 2);
        expect(call(10, 2, 0, 7), 6);
        expect(call(10, 2, 256, 7), 6);
        expect(call(11, 0, 0, 0), 4);
        expect(call(10, 2, 1, 7), 0);
        expect(call(10, 2, 1, 8), 0);
        expect(call(10, 2, 1, 9), 3);
        // B has queued messages, but even x0=2 cannot receive from its inbox.
        expect(call(11, 2, 99, 99), 4);
        finish(0, 10);
    }
    if cfg!(feature = "ipc-test-flood") {
        loop {
            expect(call(11, 0, 0, 0), 4);
        }
    }
    if cfg!(any(
        feature = "ipc-test-peer-exit",
        feature = "ipc-test-peer-fault"
    )) {
        expect(call(11, 0, 0, 0), 4);
        expect(call(10, 2, 1, 7), 5);
        finish(0, 5);
    }
    expect(call(10, 2, 1, 41), 0);
    let reply = receive();
    if reply.0 != 0 || reply.1 != 2 || reply.2 != 2 || reply.3 != 42 {
        finish(2, reply.3);
    }
    finish(0, reply.3)
}
// SAFETY: same fixed EL0 entry contract as ipc_program_a.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn ipc_program_b() -> ! {
    if cfg!(feature = "ipc-test-rejections") {
        // Yield without consuming B's mailbox, to exercise full/unauthorized receive.
        loop {
            expect(call(10, 0, 1, 0), 1);
        }
    }
    if cfg!(any(
        feature = "ipc-test-flood",
        feature = "ipc-test-peer-exit"
    )) {
        finish(0, 99);
    }
    let request = receive();
    if request.0 != 0 || request.1 != 1 || request.2 != 1 || request.3 != 41 {
        finish(3, request.3);
    }
    expect(call(10, 1, 2, request.3.wrapping_add(1)), 0);
    finish(0, request.3)
}

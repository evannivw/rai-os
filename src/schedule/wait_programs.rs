//! Linked Rust clients: a pending receive resumes once, without polling.
use super::ipc_programs::{call, expect, finish};
// SAFETY: unique linked EL0 entry, fixed executable mapping and private stack.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn wait_program_a() -> ! {
    if cfg!(feature = "wait-test-both") {
        let unexpected = call(12, 0, 0, 0);
        finish(99, unexpected.0);
    }
    if cfg!(any(
        feature = "wait-test-peer-exit",
        feature = "wait-test-peer-fault",
        feature = "wait-test-peer-spin",
        feature = "wait-test-call-limit"
    )) {
        expect(call(12, 0, 0, 0), 5);
        finish(0, 5);
    }
    if cfg!(feature = "wait-test-queued") {
        expect(call(10, 2, 1, 41), 0);
        let reply = call(12, 0, 0, 0);
        if reply.0 != 0 || reply.1 != 2 || reply.2 != 2 || reply.3 != 42 {
            finish(3, reply.0);
        }
        finish(0, reply.3);
    }
    let request = call(12, 0, 0, 0);
    if request.0 != 0 || request.1 != 2 || request.2 != 1 || request.3 != 41 {
        finish(1, request.0);
    }
    expect(call(10, 2, 2, request.3.wrapping_add(1)), 0);
    finish(0, request.3)
}
// SAFETY: same linked EL0 entry contract as wait_program_a.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn wait_program_b() -> ! {
    if cfg!(feature = "wait-test-both") {
        let unexpected = call(12, 0, 0, 0);
        finish(99, unexpected.0);
    }
    if cfg!(feature = "wait-test-peer-exit") {
        finish(0, 99);
    }
    if cfg!(feature = "wait-test-call-limit") {
        loop {
            expect(call(11, 0, 0, 0), 4);
        }
    }
    if cfg!(feature = "wait-test-queued") {
        let request = call(12, 0, 0, 0);
        if request.0 != 0 || request.1 != 1 || request.2 != 1 || request.3 != 41 {
            finish(4, request.0);
        }
        expect(call(10, 1, 2, request.3.wrapping_add(1)), 0);
        finish(0, request.3);
    }
    if cfg!(feature = "wait-test-invalid-send") {
        expect(call(10, 1, 0, 99), 6);
    }
    expect(call(10, 1, 1, 41), 0);
    let reply = call(12, 0, 0, 0);
    if reply.0 != 0 || reply.1 != 1 || reply.2 != 2 || reply.3 != 42 {
        finish(2, reply.0);
    }
    finish(0, reply.3)
}

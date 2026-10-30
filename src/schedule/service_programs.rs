//! One bounded client/worker invocation, using the existing scalar IPC ABI.
use super::ipc_programs::{call, expect, finish};
const COUNT: u64 = 20;
const RESULT: u64 = 21;
const ERROR: u64 = 22;
// SAFETY: fixed EL0 entry with its private stack; x0 is a validated scalar ID.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn record_client(id: u64) -> ! {
    let request = if cfg!(feature = "service-test-bad-request") {
        u64::MAX
    } else {
        id
    };
    let kind = if cfg!(feature = "service-test-wrong-kind") {
        23
    } else {
        COUNT
    };
    expect(call(10, 2, kind, request), 0);
    let reply = call(12, 0, 0, 0);
    if reply.0 != 0 || reply.1 != 2 || reply.2 != RESULT || reply.3 >> 32 != id {
        finish(2, reply.0);
    }
    finish(0, reply.3)
}
// SAFETY: fixed EL0 entry, private stack, no pointers received from the caller.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn record_worker() -> ! {
    let request = call(12, 0, 0, 0);
    if request.0 != 0 || request.1 != 1 {
        finish(3, request.0);
    }
    // Validate the operation before reading any source, then validate the ID.
    let result = if request.2 == COUNT {
        crate::records::count(request.3)
    } else {
        0
    };
    if result == 0 {
        expect(call(10, 1, ERROR, 1), 0);
        finish(4, 1);
    }
    let response = if cfg!(feature = "service-test-bad-reply") {
        result ^ (1_u64 << 32)
    } else {
        result
    };
    expect(call(10, 1, RESULT, response), 0);
    finish(0, result)
}

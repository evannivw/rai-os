//! A persistent EL0 worker; its local counter survives each blocked receive.
use super::ipc_programs::{call, expect, finish};
// SAFETY: fixed EL0 entry, private stack retained by the owner between requests.
// Only scalar IPC and immutable public records are accessed. No kernel pointers.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn persistent_worker() -> ! {
    let mut completed = 0_u64;
    loop {
        let request = call(12, 0, 0, 0);
        if request.0 != 0 || request.1 != 1 || request.2 != 20 {
            finish(1, 0);
        }
        if cfg!(feature = "coord-test-worker-gateway") {
            let _ = call(13, 0, 0, 0);
        }
        if cfg!(feature = "worker-test-fault") {
            // SAFETY: deliberate EL0 permission fault at the fixed kernel entry.
            // The runtime must retire this task and regain the shell.
            unsafe {
                core::arch::asm!("ldr x9, [x10]", in("x10") 0x4008_0000_u64,
                out("x9") _, options(nostack));
            }
        }
        if cfg!(feature = "worker-test-spin") {
            loop {
                core::hint::spin_loop();
            }
        }
        if cfg!(feature = "worker-test-flood") {
            loop {
                let _ = call(10, 0, 0, 0);
            }
        }
        let result = crate::records::count(request.3);
        if result == 0 {
            finish(2, 0);
        }
        // The kernel refuses another request at u64::MAX, before this addition.
        completed += 1;
        let reply = if cfg!(feature = "worker-test-bad-reply") {
            result ^ (1 << 32)
        } else {
            result
        };
        expect(call(10, 1, 21, reply), 0);
        let sequence = if cfg!(feature = "worker-test-bad-sequence") {
            0
        } else {
            completed
        };
        expect(call(10, 1, 23, sequence), 0);
    }
}

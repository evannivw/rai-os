//! A persistent EL0 worker; its local counter survives each blocked receive.
use super::ipc_programs::{call, expect, finish};
// SAFETY: fixed EL0 entry, private stack retained by the owner between requests.
// Only scalar IPC and immutable public records are accessed. No kernel pointers.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn persistent_worker() -> ! {
    let mut completed = 0_u64;
    'jobs: loop {
        let request = call(12, 0, 0, 0);
        if request.0 != 0 || request.1 != 1 || (request.2 != 20 && request.2 != 40) {
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
        if request.2 == 20 {
            expect(call(10, 1, 21, reply), 0);
        } else {
            expect(call(10, 1, 41, reply), 0);
        }
        let sequence = if cfg!(feature = "worker-test-bad-sequence") {
            0
        } else {
            completed
        };
        if request.2 == 20 {
            expect(call(10, 1, 23, sequence), 0);
        } else {
            let inject = request.3 == 3; // Public notes@2; preserve a first successful notes@1 job in tests.
            if inject && cfg!(feature = "index-stream-test-overflow") {
                let mut chunk = 0;
                while chunk < 55 {
                    expect(call(10, 1, 42, (chunk << 56) | 0x0061_6161_6161_6161), 0);
                    chunk += 1;
                }
                finish(3, 0);
            }
            let mut cursor = crate::index_cursor::Cursor::new(request.3);
            let mut word = 0_u64;
            let mut bytes = 0_u64;
            let mut chunks = 0_u64;
            let mut length = 0_u64;
            loop {
                let byte = cursor.next();
                if byte == 0 {
                    break;
                }
                if length == 384 {
                    finish(3, 0);
                }
                word |= byte << (bytes * 8);
                bytes += 1;
                length += 1;
                if bytes == 7 {
                    let number = if inject && cfg!(feature = "index-stream-test-order") {
                        chunks + 1
                    } else {
                        chunks
                    };
                    if inject && chunks == 0 && cfg!(feature = "index-stream-test-format") {
                        word = (word & !255) | b'-' as u64;
                    }
                    expect(call(10, 1, 42, word | (number << 56)), 0);
                    if inject && cfg!(feature = "index-stream-test-truncated") {
                        continue 'jobs;
                    }
                    chunks += 1;
                    word = 0;
                    bytes = 0;
                }
            }
            if bytes != 0 {
                expect(call(10, 1, 42, word | (chunks << 56)), 0);
            }
            let declared = if inject && cfg!(feature = "index-stream-test-length") {
                length + 1
            } else {
                length
            };
            expect(call(10, 1, 43, (sequence << 32) | declared), 0);
            if inject && cfg!(feature = "index-stream-test-after-end") {
                expect(call(10, 1, 42, 1), 0);
            }
        }
    }
}

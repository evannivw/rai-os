//! Built-in no_std Rust program. Its code/constants have dedicated EL0 mappings.
use core::arch::asm;
#[unsafe(link_section = ".user.rodata")]
static MESSAGE: [u8; 14] = *b"Hello from EL0";

#[inline(always)]
fn call(number: u64, first: u64, second: u64) -> (u64, u64) {
    let value;
    let error;
    // SAFETY: our integer-only ABI preserves all registers except x0/x1. The
    // kernel validates requests; no unchecked pointer is dereferenced here.
    unsafe {
        asm!("svc #0", in("x8") number, inlateout("x0") first => value,
        inlateout("x1") second => error, options(nostack));
    }
    (value, error)
}
#[inline(always)]
fn exit(code: u64) -> ! {
    // SAFETY: exit terminates this invocation and restores the saved EL1 caller.
    // If the ABI ever violates that contract, the following branch cannot return
    // through the deliberately cleared user link register.
    unsafe {
        asm!("svc #0", "2: b 2b", in("x8") 2_u64, in("x0") code,
        in("x1") 0_u64, options(noreturn));
    }
}
// SAFETY: unique entry symbol, entered only by ERET with a dedicated EL0 stack.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn user_program() -> ! {
    #[cfg(any(
        feature = "user-test-read-kernel",
        feature = "user-test-write-code",
        feature = "user-test-uart",
        feature = "user-test-execute-stack",
        feature = "user-test-privileged"
    ))]
    {
        unsafe extern "C" {
            fn user_fault_probe();
        }
        // SAFETY: test-only assembly in the EL0 text region, with C ABI scratch
        // registers. It intentionally faults; it cannot access kernel Rust data.
        unsafe { user_fault_probe() };
    }
    #[cfg(feature = "user-test-invalid-calls")]
    {
        unsafe extern "C" {
            static __user_rodata_end: u8;
        }
        expect_error(99, 0, 0, 1);
        expect_error(1, 0x4008_0000, 1, 2);
        expect_error(1, u64::MAX - 3, 8, 2);
        expect_error(1, core::ptr::addr_of!(__user_rodata_end) as u64 - 1, 2, 2);
        expect_error(1, MESSAGE.as_ptr() as u64, 65, 3);
        expect_error(1, MESSAGE.as_ptr() as u64, 0, 3);
        expect_error(1, INVALID_TEXT.as_ptr() as u64, 1, 4);
        expect_error(2, 256, 0, 3);
    }

    let (count, error) = call(1, MESSAGE.as_ptr() as u64, 14);
    exit(if count == 14 && error == 0 { 0 } else { 1 })
}

#[cfg(feature = "user-test-invalid-calls")]
#[unsafe(link_section = ".user.rodata")]
static INVALID_TEXT: [u8; 1] = [b'\n'];
#[cfg(feature = "user-test-invalid-calls")]
#[inline(always)]
fn expect_error(number: u64, first: u64, second: u64, expected: u64) {
    let (value, error) = call(number, first, second);
    if value != 0 || error != expected {
        exit(1);
    }
}

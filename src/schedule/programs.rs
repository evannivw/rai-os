//! Finite linked Rust tasks. Only the small exit boundary needs assembly.
use core::arch::asm;
#[inline(always)]
fn checksum(seed: u64, iterations: u64) -> u64 {
    let mut value = seed;
    let mut index = 0_u64;
    while index < iterations {
        value = value.wrapping_mul(33).wrapping_add(index);
        // SAFETY: opaque integer operand prevents folding the entire workload.
        // The empty assembly changes no memory, stack or flags; x9 is declared.
        unsafe {
            asm!("", inout("x9") value, options(nomem, nostack, preserves_flags));
        }
        index = index.wrapping_add(1);
    }
    value
}
#[inline(always)]
fn finish(number: u64, code: u64, value: u64) -> ! {
    // SAFETY: the lifecycle ABI consumes this SVC and never resumes a terminal
    // task. A broken return remains in EL0, where the timer can still preempt it.
    unsafe {
        asm!("svc #0", "2: b 2b", in("x8") number, in("x0") code, in("x1") value, options(noreturn));
    }
}
// SAFETY: unique ELF entry symbols; only ERET enters these user-mapped functions.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn lifecycle_program_a() -> ! {
    let value = checksum(7, 1024);
    finish(
        if cfg!(feature = "lifecycle-test-bad-call") {
            99
        } else {
            2
        },
        if cfg!(feature = "lifecycle-test-invalid-exit") {
            256
        } else {
            0
        },
        value,
    )
}
// SAFETY: same linked EL0 entry contract as lifecycle_program_a.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn lifecycle_program_b() -> ! {
    finish(2, 0, checksum(11, 4096))
}
// Public diagnostic fixture: an explicit forbidden read, no Rust reference to it.
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global lifecycle_fault",
    ".global lifecycle_fault_instruction",
    ".type lifecycle_fault, %function",
    "lifecycle_fault:",
    "mov x9, #0x40080000",
    "lifecycle_fault_instruction:",
    "ldr x0, [x9]",
    "ret",
    ".size lifecycle_fault, . - lifecycle_fault",
);

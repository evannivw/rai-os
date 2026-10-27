//! Isolated fault-injection images only. No command enables these in normal builds.
use core::arch::global_asm;

#[cfg(feature = "mmu-test-write-text")]
global_asm!(
    r#"
.section .text.mmu_test, "ax"
.global mmu_test_fault
.global mmu_fault_instruction
.type mmu_test_fault, %function
mmu_test_fault:
    ldr x0, =__text_start
mmu_fault_instruction:
    str xzr, [x0]
    ret
"#
);

#[cfg(feature = "mmu-test-write-rodata")]
global_asm!(
    r#"
.section .text.mmu_test, "ax"
.global mmu_test_fault
.global mmu_fault_instruction
.type mmu_test_fault, %function
mmu_test_fault:
    ldr x0, =__text_end
mmu_fault_instruction:
    str xzr, [x0]
    ret
"#
);

#[cfg(feature = "mmu-test-write-table")]
global_asm!(
    r#"
.section .text.mmu_test, "ax"
.global mmu_test_fault
.global mmu_fault_instruction
.type mmu_test_fault, %function
mmu_test_fault:
    ldr x0, =__page_pool_start
mmu_fault_instruction:
    str xzr, [x0]
    ret
"#
);

#[cfg(feature = "mmu-test-execute-data")]
global_asm!(
    r#"
.section .data.mmu_test, "aw"
.balign 16
.global mmu_execute_target
mmu_execute_target:
    .word 0xd65f03c0 // RET, so an incorrectly allowed fetch returns and fails the test.
.section .text.mmu_test, "ax"
.global mmu_test_fault
.global mmu_fault_instruction
.type mmu_test_fault, %function
mmu_test_fault:
    mov x9, x30
    ldr x0, =mmu_execute_target
mmu_fault_instruction:
    blr x0
    mov x30, x9
    ret
"#
);
#[cfg(feature = "mmu-test-unmapped")]
global_asm!(
    r#"
.section .text.mmu_test, "ax"
.global mmu_test_fault
.global mmu_fault_instruction
.type mmu_test_fault, %function
mmu_test_fault:
    mov x0, #0x40000000 // RAM/DTB exists physically but has no valid leaf mapping.
mmu_fault_instruction:
    ldr x1, [x0]
    ret
"#
);

pub fn run_fault() {
    let name = if cfg!(feature = "mmu-test-write-text") {
        "write-text"
    } else if cfg!(feature = "mmu-test-write-rodata") {
        "write-rodata"
    } else if cfg!(feature = "mmu-test-write-table") {
        "write-table"
    } else if cfg!(feature = "mmu-test-execute-data") {
        "execute-data"
    } else {
        "unmapped"
    };
    crate::uart::write_display(format_args!("SELF-TEST: MMU {name}\n"));
    unsafe extern "C" {
        fn mmu_test_fault();
    }
    // SAFETY: controlled assembly test under the fixed-board boot contract; no
    // Rust references to the fault targets are constructed. The MMU should trap
    // before the forbidden access. If it returns, panic marks the test as failed.
    unsafe { mmu_test_fault() };
    panic!("MMU failed to reject forbidden access");
}

//! Isolated EL0 fault probes; no Rust references are formed to forbidden memory.
#[cfg(feature = "user-test-read-kernel")]
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global user_fault_probe",
    ".global user_fault_instruction",
    "user_fault_probe:",
    "mov x9, #0x40080000",
    "user_fault_instruction:",
    "ldr x0, [x9]",
    "ret",
);
#[cfg(feature = "user-test-write-code")]
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global user_fault_probe",
    ".global user_fault_instruction",
    "user_fault_probe:",
    "ldr x9, =user_program",
    "user_fault_instruction:",
    "str xzr, [x9]",
    "ret",
);
#[cfg(feature = "user-test-uart")]
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global user_fault_probe",
    ".global user_fault_instruction",
    "user_fault_probe:",
    "mov x9, #0x09000000",
    "user_fault_instruction:",
    "ldr w0, [x9]",
    "ret",
);
#[cfg(feature = "user-test-execute-stack")]
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global user_fault_probe",
    ".global user_fault_instruction",
    "user_fault_probe:",
    "mov x9, sp",
    "user_fault_instruction:",
    "br x9",
    "ret",
);
#[cfg(feature = "user-test-privileged")]
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global user_fault_probe",
    ".global user_fault_instruction",
    "user_fault_probe:",
    "nop",
    "user_fault_instruction:",
    "mrs x0, sctlr_el1",
    "ret",
);

#[cfg(feature = "user-test-gic")]
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global user_fault_probe",
    ".global user_fault_instruction",
    "user_fault_probe:",
    "mov x9, #0x08000000",
    "user_fault_instruction:",
    "ldr w0, [x9]",
    "ret",
);

#[cfg(feature = "user-test-mask-irq")]
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global user_fault_probe",
    ".global user_fault_instruction",
    "user_fault_probe:",
    "user_fault_instruction:",
    "msr daifset, #2",
    "ret",
);

#[cfg(feature = "user-test-disable-timer")]
core::arch::global_asm!(
    ".section .user.text, \"ax\"",
    ".global user_fault_probe",
    ".global user_fault_instruction",
    "user_fault_probe:",
    "user_fault_instruction:",
    "msr cntp_ctl_el0, xzr",
    "ret",
);

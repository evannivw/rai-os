// x0: read-only kernel-owned Frame, valid for this complete synchronous call.
// Save the same 96-byte AAPCS64 caller frame used by user_return. No user address
// ever selects an EL1 continuation; slot 8/9 returns through that fixed trampoline.
.section .text.schedule_enter, "ax"
.global schedule_enter
.type schedule_enter, %function
schedule_enter:
    sub sp, sp, #96
    stp x19, x20, [sp, #0]
    stp x21, x22, [sp, #16]
    stp x23, x24, [sp, #32]
    stp x25, x26, [sp, #48]
    stp x27, x28, [sp, #64]
    stp x29, x30, [sp, #80]
    mov x30, x0               // Temporary frame pointer; restored last.
    ldr x1, [x30, #248]
    msr elr_el1, x1
    ldr x1, [x30, #256]
    msr spsr_el1, x1
    ldr x1, [x30, #280]
    msr sp_el0, x1
    ldp x0, x1, [x30, #0]
    ldp x2, x3, [x30, #16]
    ldp x4, x5, [x30, #32]
    ldp x6, x7, [x30, #48]
    ldp x8, x9, [x30, #64]
    ldp x10, x11, [x30, #80]
    ldp x12, x13, [x30, #96]
    ldp x14, x15, [x30, #112]
    ldp x16, x17, [x30, #128]
    ldp x18, x19, [x30, #144]
    ldp x20, x21, [x30, #160]
    ldp x22, x23, [x30, #176]
    ldp x24, x25, [x30, #192]
    ldp x26, x27, [x30, #208]
    ldp x28, x29, [x30, #224]
    ldr x30, [x30, #240]
    eret
.size schedule_enter, . - schedule_enter

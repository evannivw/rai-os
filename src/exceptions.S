// EL1 vectors: one EL1h probe plus bounded EL0 calls/faults; other events fatal.
.section .text.vectors, "ax"
.balign 2048
.global exception_vectors
.type exception_vectors, %object
exception_vectors:
.macro vector number
    mov x0, #\number
    b exception_entry
    .space 120
.endm
// Each group: synchronous, IRQ, FIQ, SError. Each entry is 128 bytes.
vector 0   // Current EL, SP0
vector 1
vector 2
vector 3
// Save original registers before assigning a vector number.
    b synchronous_entry
    .space 124 // Slot 4: current EL, SPx, synchronous.
    b timer_irq_entry
    .space 124 // Slot 5: current EL, SPx, IRQ.
vector 6
vector 7
    b user_synchronous_entry
    .space 124 // Slot 8: lower EL, AArch64 synchronous.
vector 9
vector 10
vector 11
vector 12  // Lower EL, AArch32 (not supported)
vector 13
vector 14
vector 15
.global exception_vectors_end
exception_vectors_end:
.size exception_vectors, . - exception_vectors

.section .text.exception_entry, "ax"
.type exception_entry, %function
exception_entry:
    msr daifset, #0xf
    // Single CPU, interrupts masked, BSS zeroed before VBAR installation.
    // A second exception during reporting halts without reusing the stack.
    ldr x6, =exception_reporting
    ldr x7, [x6]
    cbnz x7, .Lnested_halt
    mov x7, #1
    str x7, [x6]
    mrs x1, esr_el1
    mrs x2, elr_el1
    mrs x3, spsr_el1
    mrs x4, far_el1
    mov x5, sp                 // Entry SP, not a saved lower-EL user SP.
    msr spsel, #1
    ldr x6, =__exception_stack_top
    mov sp, x6                 // Dedicated 16-byte-aligned diagnostic stack.
    bl exception_report        // x0..x5 follow AAPCS64. Rust never returns.
.Lnested_halt:
    wfe
    b .Lnested_halt

.section .bss.exception_reporting, "aw", %nobits
.balign 8
exception_reporting:
    .skip 8

// The normal EL1 stack must be mapped, aligned and have 288 bytes available.
// Unlike the original fatal-only entry, this path saves a frame before switching
// stacks. Stack corruption/overflow before that save is not recoverable here.
.section .text.synchronous_entry, "ax"
.type synchronous_entry, %function
.macro save_integer_frame
    sub sp, sp, #288
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x19, [sp, #144]
    stp x20, x21, [sp, #160]
    stp x22, x23, [sp, #176]
    stp x24, x25, [sp, #192]
    stp x26, x27, [sp, #208]
    stp x28, x29, [sp, #224]
    str x30, [sp, #240]
.endm
synchronous_entry:
    save_integer_frame
    mov x20, #4
    b .Ldispatch_sync
timer_irq_entry:
    save_integer_frame
    mov x20, #5
    b .Ldispatch_sync
user_synchronous_entry:
    save_integer_frame
    mov x20, #8
.Ldispatch_sync:
    msr daifset, #0xf
    ldr x0, =exception_reporting
    ldr x1, [x0]
    cbnz x1, .Lnested_halt
    mov x1, #1
    str x1, [x0]
    mrs x0, elr_el1
    mrs x1, spsr_el1
    stp x0, x1, [sp, #248]
    mrs x0, esr_el1
    mrs x1, far_el1
    stp x0, x1, [sp, #264]
    mrs x0, sp_el0
    str x0, [sp, #280]
    mov x19, sp                // AAPCS64 preserves x19 across the Rust call.
    mov x0, sp
    ldr x1, =__exception_stack_top
    mov sp, x1
    mov x1, x20
    bl exception_dispatch     // 0: resume frame; 1: terminate user; fatal never returns.
    mov sp, x19
    cbnz x0, .Lfinish_user
    ldr x0, [sp, #280]
    msr sp_el0, x0
    ldp x0, x1, [sp, #248]
    msr elr_el1, x0            // SVC already saved the NEXT instruction address.
    msr spsr_el1, x1
    ldr x0, =exception_reporting
    str xzr, [x0]              // Permit the next sequential exception.
    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldp x8, x9, [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x19, [sp, #144]
    ldp x20, x21, [sp, #160]
    ldp x22, x23, [sp, #176]
    ldp x24, x25, [sp, #192]
    ldp x26, x27, [sp, #208]
    ldp x28, x29, [sp, #224]
    ldr x30, [sp, #240]
    add sp, sp, #288
    eret                      // Restore PC and PSTATE, including NZCV/DAIF.

.Lfinish_user:
    // Dispatch requested exit/termination, never a user-controlled return PC.
    ldr x0, =exception_reporting
    str xzr, [x0]
    add sp, sp, #288           // Back to the saved kernel caller's trampoline.
    b user_return

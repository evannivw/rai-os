// EL1 fatal-only vectors. No register restoration or ERET: never resume a fault.
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
vector 4   // Current EL, SPx (our normal EL1h execution)
vector 5
vector 6
vector 7
vector 8   // Lower EL, AArch64 (not yet supported as a runtime)
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

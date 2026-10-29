// Contract: QEMU virt, Cortex-A53, one CPU, EL1, MMU/caches off.
.section .text.boot, "ax"
.global _start
.type _start, %function
_start:
    msr daifset, #0xf           // Keep boot masked; controlled IRQ windows are enabled later.
    mrs x0, CurrentEL
    cmp x0, #4
    b.ne .Lhalt                // Unsupported boot level: do not enter Rust.
    msr spsel, #1
    ldr x0, =__stack_top
    mov sp, x0                 // AAPCS64 requires 16-byte alignment.
    ldr x0, =__bss_start
    ldr x1, =__bss_end
.Lzero:
    cmp x0, x1
    b.hs .Lrust
    str xzr, [x0], #8
    b .Lzero
.Lrust:
    // VBAR requires 2 KiB alignment. Install after stack/BSS setup, before Rust.
    ldr x0, =exception_vectors
    msr vbar_el1, x0
    isb
    bl kernel_main
.Lhalt:
    wfe
    b .Lhalt

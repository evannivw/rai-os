// Enter one built-in EL0 invocation; retain the C caller on its private EL1 stack.
.section .text.user_transition, "ax"
.global user_enter
.global user_return
.type user_enter, %function
user_enter:
    sub sp, sp, #96
    stp x19, x20, [sp, #0]
    stp x21, x22, [sp, #16]
    stp x23, x24, [sp, #32]
    stp x25, x26, [sp, #48]
    stp x27, x28, [sp, #64]
    stp x29, x30, [sp, #80]
    msr elr_el1, x0           // Kernel-selected built-in entry, never UART data.
    ldr x0, =__user_stack_top
    msr sp_el0, x0
    mov x0, #0x340             // EL0t, IRQ enabled; D/A/F remain masked.
    msr spsr_el1, x0
    // Do not expose kernel register contents/pointers to the program.
    mov x0, x1              // Validated scalar argument; zero for legacy entries.
    mov x1, xzr
    mov x2, xzr
    mov x3, xzr
    mov x4, xzr
    mov x5, xzr
    mov x6, xzr
    mov x7, xzr
    mov x8, xzr
    mov x9, xzr
    mov x10, xzr
    mov x11, xzr
    mov x12, xzr
    mov x13, xzr
    mov x14, xzr
    mov x15, xzr
    mov x16, xzr
    mov x17, xzr
    mov x18, xzr
    mov x19, xzr
    mov x20, xzr
    mov x21, xzr
    mov x22, xzr
    mov x23, xzr
    mov x24, xzr
    mov x25, xzr
    mov x26, xzr
    mov x27, xzr
    mov x28, xzr
    mov x29, xzr
    mov x30, xzr
    eret
user_return:
    msr sp_el0, xzr
    ldp x19, x20, [sp, #0]
    ldp x21, x22, [sp, #16]
    ldp x23, x24, [sp, #32]
    ldp x25, x26, [sp, #48]
    ldp x27, x28, [sp, #64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret

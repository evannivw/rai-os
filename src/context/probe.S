// AAPCS64 probe: output points to an exclusive initialized Snapshot (288 bytes).
// EL1h, one CPU, masked interrupts, valid normal stack and installed vectors.
.section .text.context_probe, "ax"
.global context_probe
.global context_probe_svc
.global context_probe_resume
.type context_probe, %function
context_probe:
    sub sp, sp, #128
    stp x19, x20, [sp, #0]
    stp x21, x22, [sp, #16]
    stp x23, x24, [sp, #32]
    stp x25, x26, [sp, #48]
    stp x27, x28, [sp, #64]
    stp x29, x30, [sp, #80]
    str x0, [sp, #96]          // Preserve output pointer before seeding x0.
    mov x2, sp
    str x2, [sp, #104]         // Independent pre-exception SP observation.
    msr nzcv, x1              // Caller supplies the tested NZCV pattern.
    mov x0, #4096
    movk x0, #0xa5a5, lsl #48
    mov x1, #4097
    movk x1, #0xa5a5, lsl #48
    mov x2, #4098
    movk x2, #0xa5a5, lsl #48
    mov x3, #4099
    movk x3, #0xa5a5, lsl #48
    mov x4, #4100
    movk x4, #0xa5a5, lsl #48
    mov x5, #4101
    movk x5, #0xa5a5, lsl #48
    mov x6, #4102
    movk x6, #0xa5a5, lsl #48
    mov x7, #4103
    movk x7, #0xa5a5, lsl #48
    mov x8, #4104
    movk x8, #0xa5a5, lsl #48
    mov x9, #4105
    movk x9, #0xa5a5, lsl #48
    mov x10, #4106
    movk x10, #0xa5a5, lsl #48
    mov x11, #4107
    movk x11, #0xa5a5, lsl #48
    mov x12, #4108
    movk x12, #0xa5a5, lsl #48
    mov x13, #4109
    movk x13, #0xa5a5, lsl #48
    mov x14, #4110
    movk x14, #0xa5a5, lsl #48
    mov x15, #4111
    movk x15, #0xa5a5, lsl #48
    mov x16, #4112
    movk x16, #0xa5a5, lsl #48
    mov x17, #4113
    movk x17, #0xa5a5, lsl #48
    mov x18, #4114
    movk x18, #0xa5a5, lsl #48
    mov x19, #4115
    movk x19, #0xa5a5, lsl #48
    mov x20, #4116
    movk x20, #0xa5a5, lsl #48
    mov x21, #4117
    movk x21, #0xa5a5, lsl #48
    mov x22, #4118
    movk x22, #0xa5a5, lsl #48
    mov x23, #4119
    movk x23, #0xa5a5, lsl #48
    mov x24, #4120
    movk x24, #0xa5a5, lsl #48
    mov x25, #4121
    movk x25, #0xa5a5, lsl #48
    mov x26, #4122
    movk x26, #0xa5a5, lsl #48
    mov x27, #4123
    movk x27, #0xa5a5, lsl #48
    mov x28, #4124
    movk x28, #0xa5a5, lsl #48
    mov x29, #4125
    movk x29, #0xa5a5, lsl #48
    mov x30, #4126
    movk x30, #0xa5a5, lsl #48
context_probe_svc:
    svc #0x5242
context_probe_resume:
    // Store every restored GPR before using any as scratch; stores/sub keep NZCV.
    sub sp, sp, #272
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
    mrs x0, nzcv
    str x0, [sp, #248]
    add x0, sp, #272
    str x0, [sp, #256]
    mrs x0, daif
    str x0, [sp, #264]
    ldr x0, [sp, #368]         // Output pointer: 272 + 96.
    mov x1, sp
    mov x2, #34               // Copy 272 bytes, including observed flags/SP.
.Lcopy_context:
    ldr x3, [x1], #8
    str x3, [x0], #8
    subs x2, x2, #1
    b.ne .Lcopy_context
    ldr x3, [sp, #376]         // Pre-exception SP: 272 + 104.
    str x3, [x0]
    str xzr, [x0, #8]
    add sp, sp, #272
    ldp x19, x20, [sp, #0]
    ldp x21, x22, [sp, #16]
    ldp x23, x24, [sp, #32]
    ldp x25, x26, [sp, #48]
    ldp x27, x28, [sp, #64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #128
    ret
.size context_probe, . - context_probe

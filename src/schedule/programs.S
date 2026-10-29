// Two deliberately non-cooperative loops with distinct arithmetic and stack words.
// ADD/STR/B preserve NZCV and x1-x30. No syscall, yield or counter polling.
.section .user.text, "ax"
.global schedule_program_a
.global schedule_program_a_end
.type schedule_program_a, %function
schedule_program_a:
.if {corrupt}
    mov x7, xzr
.endif
.Lloop_a:
    add x0, x0, #1
    str x0, [sp]
    b .Lloop_a
schedule_program_a_end:
.size schedule_program_a, . - schedule_program_a
.global schedule_program_b
.global schedule_program_b_end
.type schedule_program_b, %function
schedule_program_b:
.if {fault}
    brk #0x5243
.endif
    add x0, x0, #3
    str x0, [sp]
    b schedule_program_b
schedule_program_b_end:
.size schedule_program_b, . - schedule_program_b

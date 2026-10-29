// Linked EL0 probes. x0 is a fixed test address; x1 a public marker, never a
// kernel pointer. No stack writes except the explicit own-page access.
.section .user.text, "ax"
.balign 4
.global isolation_own
.global isolation_own_end
.type isolation_own, %function
isolation_own:
    str x1, [x0]
    ldr x1, [x0]
    mov x0, #0
    mov x8, #2
    svc #0
1:  b 1b
isolation_own_end:
.size isolation_own, . - isolation_own

.global isolation_read
.global isolation_read_fault
.global isolation_read_end
.type isolation_read, %function
isolation_read:
isolation_read_fault:
    ldr x1, [x0]
    mov x0, #0
    mov x8, #2
    svc #0
1:  b 1b
isolation_read_end:
.size isolation_read, . - isolation_read

.global isolation_write
.global isolation_write_fault
.global isolation_write_end
.type isolation_write, %function
isolation_write:
isolation_write_fault:
    str x1, [x0]
    mov x0, #0
    mov x8, #2
    svc #0
1:  b 1b
isolation_write_end:
.size isolation_write, . - isolation_write

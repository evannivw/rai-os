# First lower-privilege Rust program

Implemented October 28, 2026. `user.run` enters a built-in Rust program at EL0,
accepts two bounded syscalls and returns to the kernel after exit or a contained
synchronous user fault. The later [deadline increment](USER_TIMEOUT.md) also
terminates expired invocations. The MMU denies this program access to kernel memory and
UART. This is one privilege boundary in one address space, not a multi-process
runtime, application loader, scheduler or complete microkernel.

## Try it

```sh
sh scripts/console.sh
```

Enter `user.run`, then `context.check` or `echo still running`. The normal result:

```text
mode=EL0; state=exited; code=0; syscalls=2; rejected=0; output=Hello from EL0
```

The program in `src/user/program.rs` requests a write, checks the returned count
and status, then exits. Code is Rust with tiny inline assembly syscall stubs.
The write copies text to a private kernel result buffer; the console publishes
it after completion, preserving structured UART framing. It is not direct UART
access or a full POSIX write implementation. The native panel is unchanged.

`context.check` is a diagnostic command, not an application service. `user.run`
is a kernel console command that starts our built-in program. The program itself
uses numeric SVC calls, not shell strings or the host UART protocol.

## Small syscall ABI

AArch64 `svc #0`, with x8 selecting the operation. x0/x1 contain arguments; on a
returning call they contain value/error respectively. Zero error means success.
Other general registers, NZCV and SP are restored. SIMD/FPU is unused/unsupported.

| x8 | Operation | Inputs | Outcome |
| --- | --- | --- | --- |
| 1 | Write result text | x0=address, x1=length | Byte count / zero, or zero / error |
| 2 | Exit invocation | x0=code in 0..255 | Never returns to this program |

Writes accept 1..64 bytes wholly within either the user constant region or user
stack. Address addition is checked for overflow. The kernel copies into a bounded
local buffer before validating printable ASCII (space allowed; control characters
rejected). No bytes are committed on error. Cumulative output is limited to 64
bytes per invocation. No user-memory reference escapes the handler, and there is
no concurrent user execution or remapping during the copy.

Errors: 1=unknown call, 2=invalid address/range, 3=invalid length or exit code,
4=invalid text, 5=output buffer full. Unknown numbers and invalid arguments return
errors so the program can continue. No raw kernel operations or host API calls
are exposed. This ABI is experimental, not a compatibility promise.

## Memory and execution

The ELF links dedicated `.user_text`, `.user_rodata` and an 8 KiB user stack after
the kernel's exception stack. All permission boundaries are page aligned. A 4 KiB
unmapped guard page lies on each side of the user stack. These reservations are
outside the physical-page pool; current counts are 256 total, twelve table pages,
244 free after the later private-task-map increment. The initial mapped window still ends below 0x40200000.

| Region | EL0 | EL1 |
| --- | --- | --- |
| Kernel code/data/stacks/tables and UART/GIC | No access | Existing kernel permissions |
| User instructions | Read / execute | Read only, no execution |
| User constants | Read only, no execution | Read only, no execution |
| User stack | Read / write, no execution | Read / write, no execution |
| User stack guards | Unmapped | Unmapped |

The mappings are immutable and shared in a single translation context. Each run
clears the user stack and creates a fresh result/session. The transition saves
kernel callee-saved registers on SP_EL1, assigns SP_EL0, clears x0-x30, and uses
ERET to enter EL0t with IRQ enabled and D/A/F masked (0x340). No kernel register values
are intentionally passed to the program.

Vector 8 captures user registers on the private EL1 stack, including SP_EL0, then
runs dispatch on the exception stack. A syscall is accepted only with the expected
SVC syndrome, EL0t/IRQ-enabled saved state, return PC inside user code and an aligned
SP inside the user stack. Write returns restore the frame with new x0/x1 values.
Exit/fault/timeout returns use a fixed kernel trampoline; user input cannot select its PC.
The original EL1 context probe and kernel fatal path are retained.

One private AtomicPtr identifies the stack-owned active Session. `run` publishes
it only for the synchronous invocation and clears it before moving the report.
Dispatch borrows it exclusively while the caller is suspended; the assembly
nested-entry guard and single-CPU/masked-interrupt contract exclude parallel or
recursive mutation. No allocation, async callback or user-accessible session
pointer is involved. Atomics do not make the rest of this runtime SMP-safe.

## Faults and limits

An EL0 synchronous fault records syndrome, PC, raw FAR, saved status and user SP,
terminates this invocation and returns to the console. Its payload says
`state=faulted`; this is not a successful program result. UART `ok` means the
run request was handled and a report is available: callers must inspect state
and exit code. Kernel exceptions still halt the guest unless they are the exact
approved EL1 diagnostic SVC or timer IRQs. FAR validity depends on the exception class.

The timer owner now gives each invocation a fixed 100 ms guest-counter deadline.
`user.spin` demonstrates termination of a loop that never calls the kernel.
See [USER_TIMEOUT.md](USER_TIMEOUT.md) for IRQ/syscall expiry and limits. This is
termination, not suspension/resumption or multi-task scheduling; the console and
cooperative task do not progress during the invocation.
Only the built-in image and explicit test variants are accepted, not arbitrary
binaries. There is no user heap, filesystem, dynamic linker, independent process
address space, multitasking, phone support or agent migration in this increment.
Kernel stack corruption/nested faults remain outside the tested recovery scope.

## Verification

```sh
python3 scripts/verify_user.py
python3 scripts/verify_user.py --release
```

Verified with Rust 1.90.0 and QEMU 11.1.2 on macOS ARM64. Each profile now boots ten
images: normal, invalid calls, and eight hardware fault probes (including GIC
access and attempts to mask IRQs or disable CNTP). Every image runs
the program twice and checks timer/context/echo continuity, unchanged MMU status,
physical-page diagnostics and malformed UART argument rejection. The normal image
also runs through the direct shell. ELF checks independently validate sections,
stack/guard placement and test PCs.

The invalid-call program checks eight error replies (unknown number, kernel
pointer, overflowing span, boundary crossing, oversized/empty text, control byte,
invalid exit code), then writes/exits normally. Fault probes attempt kernel read,
code write, UART/GIC reads, stack execution, a privileged register read, DAIFSet
and a CNTP_CTL write. The verifier
checks exact syndromes and fault PCs/addresses where valid, saved EL0 state, user
SP bounds and successful subsequent kernel commands. Pure tests cover range
validation and the updated mapping permissions/layout rejection.

Test features are mutually exclusive with other test features and build into
`target/user-self-test/<case>`. Normal images have no fault-probe symbols.
Original seven-case captures: [debug](evidence/2026-10-28-user-debug.txt) and
[release](evidence/2026-10-28-user-release.txt). Regression results are recorded in
[WORKLOG.md](WORKLOG.md). Cumulative output exhaustion, corrupted/nested kernel
stacks and SMP were not injected by these tests. Separate timer/GIC evidence is
linked from [TIMER.md](TIMER.md); deadline and timer-access evidence is in
[USER_TIMEOUT.md](USER_TIMEOUT.md).

References: [Arm memory permissions](https://documentation-service.arm.com/static/63a43e333f28e5456434e18b)
and [AArch64 exception model](https://documentation-service.arm.com/static/63a065c41d698c4dc521cb1c).

The October 28 timer increment initially used six table
pages and 250 free pool pages. Earlier evidence files retain the actual counts
from their original runs. The additional L3 table maps the GIC device pages.

The separate [scheduling diagnostic](SCHEDULING.md) reuses the user RAM region
only while this invocation is inactive. It splits the existing stack into two
halves with shared EL0 permissions; normal user.run still clears and owns all
8 KiB. No mappings or syscalls changed with that diagnostic.

The later [task lifecycle](TASK_LIFECYCLE.md) also owns this region exclusively
while running. Its linked tasks use a separate, intentionally smaller exit-only
ABI, returning a scalar in x1; the legacy user.run write/exit ABI is unchanged.

Current [task maps](TASK_MEMORY.md) use twelve table pages and 244 free pool
pages. Legacy user.run/user.spin still use the base map with the full 8 KiB stack;
scheduling owns the region exclusively and uses separate A/B writable pages.
Previous counts and shared-map descriptions above describe earlier increments.

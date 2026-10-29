# Private task stack mappings

Implemented October 28, 2026. The two fixed EL0 tasks now have hardware-enforced
private writable stack pages. A cannot read/write B's stack, and B cannot
read/write A's stack, under their respective task maps. Shared user code and
constants remain read-only. This is a memory-protection foundation for guest
services, not a complete process security model or a performance result.

## What changes during a task switch

The kernel retains task records in its protected RAM as described in
[TASK_LIFECYCLE.md](TASK_LIFECYCLE.md). Alongside saving/restoring CPU registers,
the shared run_turn adapter now selects one of two fixed page-table roots.
Task identity selects the root through the typed TaskSpace A/B boundary; callers
cannot supply arbitrary roots or mapping entries.

| Region | Base map | Task A at EL0 | Task B at EL0 |
| --- | --- | --- | --- |
| User code | EL0 read/execute | Read/execute | Read/execute |
| User constants | EL0 read only | Read only | Read only |
| First 4 KiB stack page | EL0 read/write | Read/write | No access |
| Second 4 KiB stack page | EL0 read/write | No access | Read/write |
| Kernel data, stacks, pool, tables, devices | EL1 only | No access | No access |

Writable user pages are non-executable. Both outer user-stack guards remain
unmapped; the peer page acts as a permission boundary between adjacent stacks.
Physical addresses and memory types do not change between roots. The kernel
keeps access to both pages, so it can save state, validate results and initialize
new invocations. A privileged kernel bug can still defeat these protections.

The legacy single-program user.run/user.spin commands keep the base map and full
8 KiB user stack. They never run concurrently with the scheduler. Every new
invocation clears its user RAM. Only the two fixed tasks exist; there is no
persistent application loader, private heap/data allocator or dynamic map API.

## Immutable tables and switching contract

Twelve owned allocator pages (48 KiB) hold the maps:

- Pages 0-5: original base root, low L2, UART L3, RAM L2, RAM L3, GIC L3.
- Pages 6-8: A root, RAM L2, RAM L3.
- Pages 9-11: B root, RAM L2, RAM L3.

Task roots share the low/device tables. Their RAM leaves equal the base leaves
except that the peer stack leaf denies EL0. All twelve table pages are read-only
and non-executable in every map. Construction finishes with the MMU off; table
contents are never edited or freed after activation. The 256-page pool therefore
has 244 pages left, rather than the prior 250. These counts describe our reserved
1 MiB pool, not total QEMU RAM consumption.

with_task verifies the base root and masked interrupts, installs the selected
root, runs a bounded continuation and restores the base root before returning.
The continuation's Result is retained until restoration, so both success and
ordinary error paths restore the base. Panic and unexpected EL1 exceptions halt
the guest rather than unwind. After restoration, root readback and interrupt-mask
checks must pass. The UART report is produced only in the base map.

All roots use ASID 0. Each root change executes DSB SY, writes TTBR0_EL1, ISB,
TLBI VMALLE1, DSB SY and ISB. The full local invalidation includes global entries;
permissions from the previous task must not remain usable at the next EL0 entry.
Kernel code, stack, vectors and devices retain identical mappings throughout the
switch. No Rust reference points into a mapping that disappears. This deliberately
simple approach avoids an ASID allocator but adds switching overhead; it is not
an optimization claim. The contract is one CPU with interrupts masked in kernel
Rust, caches off and no DMA or competing memory owner. SMP, ASID reuse, dynamic
permission changes and cacheable remapping need different, reviewed contracts.

Arm's [memory-management guide](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/LearnTheArchitecture-MemoryManagement-101811_0100_00_en.pdf)
explains page tables, task views of memory, ASIDs and translation-cache maintenance.

## Try the hardware checks

```sh
sh scripts/console.sh
```

Then enter:

```text
schedule.isolate
schedule.run
schedule.check
echo still running
```

schedule.isolate runs two rounds. It fills A/B's entire pages with distinct public
marker bytes. Linked assembly probes first perform permitted own-page writes and
reads, then attempt peer reads and writes in both directions. This warms own-page
translations before task switches, exercising stale-permission risks. Each
rejected access must report the exact EL0 data-abort syndrome, instruction and
address. Complete page checks verify that marker bytes remain intact before the
victim accesses its own page again, so its marker write cannot hide corruption. Probes do not otherwise write
stack memory, so every byte can be checked. Timer preemption can resume a checked
probe in the same task map; a sixth preemption aborts this diagnostic.

A successful report starts with:

```text
rounds=2; reads_denied=4; writes_denied=4; own_accesses=12; memory_unchanged=true; base_restored=true
```

It then contains actual final-round ESR/PC/FAR tuples for A-read, A-write, B-read
and B-write. Both rounds are checked internally. Read ESR is 0x9200000f; write ESR
is 0x9200004f. A faulting instruction is never resumed. The report flags only appear
after all checks pass. No addresses, executable bytes or permissions are accepted
from UART. [PROTOCOL.md](PROTOCOL.md) specifies fields and error semantics.

The finite Rust tasks still exit normally. schedule.check still verifies real
IRQ pause/resume, all saved integer registers, flags and per-task stack markers;
its turns now use the private maps too. A task exception in schedule.run/fault
still lets its ready peer continue through the existing lifecycle rules.

## Verification and limits

```sh
python3 scripts/verify_isolation.py
python3 scripts/verify_isolation.py --release
```

Each profile runs four pure table tests. The new exhaustive task-leaf comparison
checks that only the peer permission differs, only the selected stack page is
EL0 writable, all table pages stay protected, and no aliases are introduced.
Invalid layouts are rejected before mutating buffers.

Real QEMU tests invoke schedule.isolate four times in the normal image: three
structured requests and one direct shell command. That is 32 denied accesses and
48 successful own-page accesses per profile. The host independently checks saved
fault syndromes and PC/FAR against ELF symbols. Subsequent lifecycle, register
scheduling, legacy user, memory and echo operations check continued operation;
MMU snapshots before/after must match. Invalid arguments are rejected.

A separate isolation-test-open-peer image deliberately retains both writable EL0
stack mappings. All three diagnostic invocations must return
isolation_expected_fault rather than report protection. Subsequent commands and
base-root readback check recovery after this early error. The feature is disabled
by default, mutually exclusive with other diagnostic features and built in its
own target directory. Never deploy that image as evidence of protection.

Actual output: [debug](evidence/2026-10-28-isolation-debug.txt) and
[release](evidence/2026-10-28-isolation-release.txt). Earlier evidence and page
counts remain unchanged in historical captures. Regression results are recorded
in [WORKLOG.md](WORKLOG.md).

This verifies the supported QEMU board with fixed linked tasks. It does not claim
protection from kernel bugs, side channels, DMA, hardware faults, all malicious
programs, SMP races or full stack exhaustion. Code is shared; there is no private
executable image per task. Kernel modules remain mutually trusted at EL1. A guest
coordinator, agents, phone support and Meta integration remain pending. The next
foundation is bounded message exchange with explicit ownership and validation,
not direct reads of another task's data.

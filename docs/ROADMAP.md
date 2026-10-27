# From a booting kernel to guest services

Status: October 27, 2026. Each step should produce a small, independently
observable result before the next. The kernel supplies mechanisms and enforces
access; a future coordinator chooses policy above it. Current Rust modules are
not isolated processes. The existing coordinator still runs on the host.

| Step | Status | What it enables / how to observe it |
| --- | --- | --- |
| Boot, UART, bounded commands | Done | QEMU boots our Rust kernel and returns real results. |
| Clock and cooperative job | Done | Measured uptime, word-count progress and cancellation. |
| Fatal exception diagnostics | Done | A deliberate fault reports its cause and stops safely within the tested conditions. |
| Physical-page ownership | Done | Allocate, zero, test and release a reserved pool without overlapping kernel regions. |
| Translation tables and permissions | Done in this increment | MMU rejects forbidden writes, instruction fetches and unmapped reads. |
| Resumable exception context | Next proposed increment | Save required CPU state, handle one controlled event and restore execution without corruption. |
| One lower-privilege program and bounded syscall | Planned | An EL0 program requests a kernel operation and resumes; forbidden access is denied. Requires explicit service memory ownership/mappings. |
| Timer interrupt and scheduling | Planned | Save/switch execution contexts; one service cannot monopolize the CPU. Requires interrupt-controller setup and synchronization review. |
| Bounded IPC and service lifecycle | Planned | Services exchange validated messages; ownership and fault policy are explicit. |
| Guest coordinator and executors | Planned | Move decision/execution services into RAI-OS with enforced permissions. Adapt host std/storage assumptions. |
| Meta-driven policy | Planned | Real model proposals go through validated operations and produce observed outcomes. |

This is a dependency roadmap, not a promise to finish every row before the
hackathon. EL0/scheduling/IPC require careful validation; a working demo can
integrate Meta through the host bridge before all service infrastructure exists.
The API credentials and HTTPS stay on the host for the initial demo. Revisit
scope against [HACKATHON_PLAN.md](HACKATHON_PLAN.md) before expanding a phase.

## What runs where today

- QEMU emulates the machine. Our own kernel runs inside it; this is not a Linux
  kernel or an Android derivative. It currently targets this virtual board only.
- The terminal and Device Lab panel run on the Mac. They send UART requests and
  display guest results. Neither is necessary for direct guest UART operation.
- The host coordinator chooses the next bounded word-count job. It is a temporary
  development service, not a guest process and not connected to Meta yet.
- A guest-native graphical shell, physical-phone drivers, Android app compatibility,
  storage, networking and measured performance gains remain outside this increment.

## How this reaches the demo

Keep the kernel route and demo route distinguishable. Once the needed execution
operations exist, resume the [demo result contract](DEMO_ACCEPTANCE.md), implement
its independent verifier, establish the deterministic baseline, integrate an
actual eligible Meta model, and compare the same workload and constraints.
Record model/network overhead and unsuccessful outcomes as well as successes.
The visual panel can show that evidence; it does not manufacture it or establish
that AI is superior. Retain dates and eligible contributions, then prepare the
three-minute video and reproducible checkout instructions.

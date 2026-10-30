# EL1 translation tables and permissions

Implemented October 27, 2026. The kernel enables the AArch64 stage-1 MMU before
printing its normal boot banner. The October 28 private-stack increment adds two
fixed task roots beside the original base root. All maps remain identity-mapped:
a mapped virtual address has the same physical address. This is a foundation for
protection, not isolated services, a complete microkernel, or an efficiency result.

## Try and verify

```sh
sh scripts/console.sh
```

Enter `mmu.info`, `memory.info`, and `memory.check`. The first reads translation
registers; the others show 256 total pages, twelve reserved for tables, and 244
available pages. A memory check leaves the table pages and mappings intact.
The graphical host panel is optional and unchanged.

```sh
python3 scripts/verify_mmu.py
python3 scripts/verify_mmu.py --release
```

Each run executes four pure table-construction tests, a normal QEMU boot with seven
structured exchanges and one shell query, then five separate intentional-fault
boots. `--case write-text` (or another case below) narrows the fault selection;
it still runs the pure tests and normal boot.

| Injected operation | Expected architectural result |
| --- | --- |
| Write kernel instructions | EL1 data abort, level-3 permission fault |
| Write read-only constants | EL1 data abort, level-3 permission fault |
| Write root translation table | EL1 data abort, level-3 permission fault |
| Execute an instruction in writable data | EL1 instruction abort, level-3 permission fault |
| Read unmapped RAM at 0x40000000 | EL1 data abort, level-3 translation fault |

Verified with Rust 1.90.0 and QEMU 11.1.2 on macOS ARM64.
Debug and release passed all five injections. The verifier checks exception class,
status, exact fault PC/address against ELF symbols, saved EL1h/masked state, both
stack ranges/alignment, fatal output and absence of further command responses.
QEMU stays alive while the guest halts; the verifier then terminates/reaps it.
These are emulated hardware permission checks, not checks implemented in the
command parser. Captures: [debug](evidence/2026-10-27-mmu-debug.txt) and
[release](evidence/2026-10-27-mmu-release.txt).

Fault features are disabled by default and mutually exclusive, including
`exception-self-test`. Do not use `--all-features`. The verifier uses separate
`target/mmu-self-test/<case>` directories, so normal panel/console images keep
working. Intentional accesses are assembly test instructions; no Rust references
to forbidden targets are constructed. Normal ELF inspection rejects test triggers.

## Mapping policy

The fixed board remains virt-9.2, Cortex-A53, TCG, one CPU, 128 MiB and EL1. We use
4 KiB pages, a 39-bit TTBR0 virtual range starting at level 1, 32-bit physical
addresses, and disabled TTBR1 walks. No address randomization or aliases.

| Mapped region | EL1 permission | Memory type |
| --- | --- | --- |
| Kernel text, including vectors | Read / execute | Normal non-cacheable |
| Constants and twelve table pages | Read only, no execution | Normal non-cacheable |
| Data, BSS, both stacks, remaining pool | Read / write, no execution | Normal non-cacheable |
| Two GIC pages at 0x08000000 and 0x08010000 | Read / write, no execution | Device-nGnRnE |
| One PL011 page at 0x09000000 | Read / write, no execution | Device-nGnRnE |
| User code/constants | Read only, no execution at EL1 | Normal non-cacheable |
| User stack | Read / write, no execution | Normal non-cacheable |

The original mapping denied EL0 on every leaf. The subsequent
[EL0 increment](USER_PROGRAM.md) adds explicit user regions; kernel/device leaves
still deny EL0 and set unprivileged execute-never. WXN additionally
forbids execution of writable mappings. Instruction/data caches remain disabled;
this increment makes no speed or battery claim.

In the default headless build, only RAM from 0x40080000 through the exclusive pool end is mapped. The linker
requires the entire kernel/stacks/pool to fit below 0x40200000. Low RAM containing
the DTB, the rest of the 128 MiB RAM, and other device ranges are unmapped. This is
a deliberate small initial window, not general device discovery. All section
permission boundaries are 4 KiB aligned. Kernel stacks still lack guard pages;
the later user stack has an unmapped guard page on each side.

Twelve allocator-owned pages (48 KiB) hold the original six-page base tree plus
a private root/RAM L2/RAM L3 for each of A and B. Device tables are shared. All
roots preserve kernel mappings; each task root removes EL0 access to its peer
stack page. Unused entries remain invalid. The MMU owner keeps these
non-copy Page tokens for the rest of execution. They cannot be returned through
any command, and their mapped contents become read-only after activation.

## Code and safety boundaries

- `src/mmu/tables.rs` builds descriptors using only bounded arrays and integers.
  It validates layout/alignment and distinct table addresses before mutation.
- `src/memory/mod.rs` supplies zeroed owned pages. Its unsafe table view requires
  an exclusive writable page before activation, and its borrow ends before RO.
- `src/mmu/mod.rs` constructs tables once with cold MMU/caches and exclusive RAM.
  Its later fixed-root switch requires one masked EL1 CPU and unchanged kernel
  mappings. It flushes local EL1 translations on every root change and restores
  the base root after a bounded continuation, including an error result.
- `linker.ld` aligns permission boundaries and checks the initial window.
- `src/mmu/self_test.rs` contains opt-in assembly fault probes only.
- `scripts/verify_mmu.py` observes QEMU and independently checks ELF boundaries
  and fault reports. It does not infer success from compilation.

Activation preserves reset/reserved SCTLR bits, selects little-endian operation,
and programs MAIR/TCR/TTBR registers. Barriers publish table writes, invalidate
old translations, and synchronize the switch. The live PC, SP, exception code,
diagnostic stack and UART retain identity mappings throughout. Table walks and
normal RAM are non-cacheable. No runtime table mutation or arbitrary remapping
API is provided. Fixed-root switching and its translation-cache maintenance
contract are documented in [TASK_MEMORY.md](TASK_MEMORY.md).

Permissions catch selected illegal accesses; privileged kernel code can still
change MMU registers. Kernel modules still share EL1 privilege. The later built-in EL0 program has
restricted mappings, write/exit syscalls and contained synchronous faults. There
are no dynamically created service address spaces, per-task ASIDs, demand paging
or resumption of faulting user instructions. Unexpected kernel faults still halt the whole guest.
The next steps are in [ROADMAP.md](ROADMAP.md).

Architectural references:

- [Arm memory management](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/LearnTheArchitecture-MemoryManagement-101811_0100_00_en.pdf): translation stages, tables and permissions.
- [Arm memory model](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/Armv8-A%20memory%20model%20guide.pdf): memory attributes and ordering.

The October 28 timer increment used six tables and 250 free pages. Current
[task-map ownership](TASK_MEMORY.md) uses twelve tables and 244 free pages.
Earlier evidence files retain the actual counts from their original runs.


## Optional native display mapping — October 30

The guest-display feature adds an EL1-only Device page at 0x09020000 for fw_cfg
and one normal non-cacheable EL1-only RW/NX L2 block at 0x41000000..0x41200000.
The linker reserves that entire 2 MiB separately from the kernel/page pool. All
three roots share the mappings, with no alias or extra table-page allocation.
EL0 read/write/execute attempts on display RAM and access to fw_cfg were tested
as real hardware faults. See [GUEST_DISPLAY.md](GUEST_DISPLAY.md) for the adapter
contract and graphics-specific tests. The default build keeps the original map.

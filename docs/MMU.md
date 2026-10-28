# EL1 translation tables and permissions

Implemented October 27, 2026. The kernel enables the AArch64 stage-1 MMU before
printing its normal boot banner. This is one identity-mapped EL1 address space:
a mapped virtual address has the same physical address. It is a foundation for
protection, not isolated services, a complete microkernel, or an efficiency result.

## Try and verify

```sh
sh scripts/console.sh
```

Enter `mmu.info`, `memory.info`, and `memory.check`. The first reads translation
registers; the others show 256 total pages, five reserved for tables, and 251
available pages. A memory check leaves the table pages and mappings intact.
The graphical host panel is optional and unchanged.

```sh
python3 scripts/verify_mmu.py
python3 scripts/verify_mmu.py --release
```

Each run executes two pure table-construction tests, a normal QEMU boot with seven
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
| Constants and five table pages | Read only, no execution | Normal non-cacheable |
| Data, BSS, both stacks, remaining pool | Read / write, no execution | Normal non-cacheable |
| One PL011 page at 0x09000000 | Read / write, no execution | Device-nGnRnE |
| User code/constants | Read only, no execution at EL1 | Normal non-cacheable |
| User stack | Read / write, no execution | Normal non-cacheable |

The original mapping denied EL0 on every leaf. The subsequent
[EL0 increment](USER_PROGRAM.md) adds explicit user regions; kernel/device leaves
still deny EL0 and set unprivileged execute-never. WXN additionally
forbids execution of writable mappings. Instruction/data caches remain disabled;
this increment makes no speed or battery claim.

Only RAM from 0x40080000 through the exclusive pool end is mapped. The linker
requires the entire kernel/stacks/pool to fit below 0x40200000. Low RAM containing
the DTB, the rest of the 128 MiB RAM, and other device ranges are unmapped. This is
a deliberate small initial window, not general device discovery. All section
permission boundaries are 4 KiB aligned. Kernel stacks still lack guard pages;
the later user stack has an unmapped guard page on each side.

Five allocator-owned pages (20 KiB) hold one L1 root and two L2/L3 branches: one
for UART, one for RAM. Unused entries remain invalid. The MMU owner keeps these
non-copy Page tokens for the rest of execution. They cannot be returned through
any command, and their mapped contents become read-only after activation.

## Code and safety boundaries

- `src/mmu/tables.rs` builds descriptors using only bounded arrays and integers.
  It validates layout/alignment and distinct table addresses before mutation.
- `src/memory/mod.rs` supplies zeroed owned pages. Its unsafe table view requires
  an exclusive writable page before activation, and its borrow ends before RO.
- `src/mmu/mod.rs` composes tables and writes architectural registers once. It
  requires one EL1 CPU, masked interrupts, cold MMU/caches and exclusive RAM.
- `linker.ld` aligns permission boundaries and checks the initial window.
- `src/mmu/self_test.rs` contains opt-in assembly fault probes only.
- `scripts/verify_mmu.py` observes QEMU and independently checks ELF boundaries
  and fault reports. It does not infer success from compilation.

Activation preserves reset/reserved SCTLR bits, selects little-endian operation,
and programs MAIR/TCR/TTBR registers. Barriers publish table writes, invalidate
old translations, and synchronize the switch. The live PC, SP, exception code,
diagnostic stack and UART retain identity mappings throughout. Table walks and
normal RAM are non-cacheable. No runtime table mutation, remapping or cache
maintenance API is provided. Those would need new ordering/ownership contracts.

Permissions catch selected illegal accesses; privileged kernel code can still
change MMU registers. Kernel modules still share EL1 privilege. The later built-in EL0 program has
restricted mappings, write/exit syscalls and contained synchronous faults. There
are no separate service address spaces, ASIDs, demand paging or resumption of
faulting user instructions. Unexpected kernel faults still halt the whole guest.
The next steps are in [ROADMAP.md](ROADMAP.md).

Architectural references:

- [Arm memory management](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/LearnTheArchitecture-MemoryManagement-101811_0100_00_en.pdf): translation stages, tables and permissions.
- [Arm memory model](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/Armv8-A%20memory%20model%20guide.pdf): memory attributes and ordering.

# First physical-page allocator

Implemented October 27, 2026. The kernel manages an explicitly reserved 1 MiB pool:
256 pages of 4096 bytes. This module supplies physical-page ownership and zeroing,
not a Rust heap allocator, process isolation, or all available system RAM.
The subsequent [MMU layer](MMU.md) now owns five pages permanently (20 KiB).
The existing word-count job still owns its fixed inline array; it has not been
migrated to page-backed storage. Consumers are the MMU tables and a bounded diagnostic.

## Try it

From the repository root:

```sh
sh scripts/console.sh
```

Then enter these commands in that console (also available in direct guest UART):

```text
memory.info
memory.check
memory.info
```

`memory.info` reports the pool's page size, total/free/used counts, and linked
start/end addresses. Addresses may differ between debug and release or builds.
These are actual allocator counters, not host RAM usage or total guest free RAM.

`memory.check` uses only currently free pages; if none are free, it returns
`memory_busy` without allocating. It allocates the 251 available pages, checks every byte
is zero, writes distinct markers at each page's two ends, then verifies all
markers after all allocations. An extra allocation must fail. It releases all
diagnostic-owned pages, allocates the first of those again, verifies that its old contents were cleared,
and releases it. Success reports:

```text
checked_pages=251; exhausted=true; zeroed=true; reused=true; free_pages=251
```

Those flags are produced only after the checks execute. An invariant violation
panics/halts rather than returning a success report. This diagnostic runs
synchronously and temporarily delays UART processing and cooperative task steps;
it is not background telemetry or an efficiency benchmark. No raw-address
allocation/free/read/write commands are exposed. The graphical panel has no new
memory controls in this increment; its existing task/coordinator views still work.

## Reserved layout

The supported configuration remains QEMU virt-9.2, Cortex-A53, one CPU, 128 MiB,
MMU active with caches off after early initialization. Physical RAM spans [0x40000000, 0x48000000). The ELF kernel starts at
0x40080000, leaving the low region for boot data. QEMU documents the DTB at the
start of RAM for this bare-metal ELF boot path. The allocator does not parse it
or discover arbitrary board memory; do not change the board/RAM/firmware contract
without revisiting reservations.

| Region | Allocation policy |
| --- | --- |
| Boot data below 0x40080000 | Excluded from the pool |
| Kernel text, vectors, read-only data, data, BSS | Excluded |
| 64 KiB normal stack and 16 KiB exception stack | Excluded |
| Alignment padding up to the next 4096-byte boundary | Excluded |
| `.page_pool`, next 1 MiB | Managed as 256 pages |
| Remaining QEMU RAM | Unmanaged; not reported as free |

The linker places `.page_pool` as NOLOAD after both stacks and exports its start
and exclusive end. Assertions check alignment, non-overlap with the preceding
kernel regions, and the initial MMU window limit (0x40200000). The verifier independently reads
ELF symbols and compares the live pool addresses against them. No page is handed
out until its contents have been explicitly zeroed; NOLOAD is not a promise of
initialized bytes. Zeroing is performed at allocation, not at release, and is
not a complete security boundary while all kernel code shares privilege/memory.

## Responsibilities and ownership

- `src/memory/frames.rs`: pure bounded address bookkeeping, using four u64 words
  as a 256-bit occupancy map. Allocation picks the first free page. Invalid ranges,
  unaligned/out-of-range frees, and already-free pages are rejected. This module
  does not dereference addresses or allocate host/guest heap memory.
- `src/memory/mod.rs`: sole physical adapter, initialized once by kernel_main.
  Allocation marks a page used, zeroes 4096 bytes, and returns a private-field
  Page token. Tokens are neither Copy nor Clone. Release consumes the token;
  an unsafe borrowed table view is available only before MMU activation. Dropping a token without
  release leaks capacity instead of permitting duplicate ownership. The internal
  numeric bitmap interface alone is not a generation-aware stale-pointer API;
  the token adapter supplies the normal-code ownership discipline.
- `src/command.rs`: delegates allowed info/check operations to the owned Memory
  instance. `console.rs` transports their results. Host adapters only encode
  commands; they neither allocate guest pages nor manufacture pool counters.

Unsafe construction requires exclusive ownership of the whole writable linker
region, exactly once per boot, with no competing allocator, firmware, DMA, or
stack usage. The mutable external start symbol describes the entire reserved
array; symbol addresses are taken without reading uninitialized bytes. Page
clearing and diagnostic volatile accesses are confined to live owned pages.
Metadata is accessed through exclusive mutable borrows on the single CPU; no
mutable global allocator or synchronization framework was added. SMP, concurrent
drivers, mutable MMU mappings, multiple pools, and resumable interrupt access would need
new ownership and synchronization contracts.

## Verification

```sh
python3 scripts/verify_memory.py
python3 scripts/verify_memory.py --release
```

Each run executes three pure frame tests and 18 real-QEMU exchanges (15 structured,
three direct shell). Tests cover unique allocation across bitmap-word boundaries,
exhaustion/reuse, invalid/double frees without state mutation, invalid ranges and
a 65-page partial bitmap, actual RAM zeroing and markers, repeated diagnostics,
invalid operation arguments, and task/echo operation after memory checks.

The memory_busy branch guards complete exhaustion; five pages stay reserved for
tables, but no current UART operation can reserve the remaining pages permanently,
so that branch is not exercised by the live command test. The page pool is not exhaustively tested against corrupt metadata,
faulty hardware, concurrent callers, or adversarial privileged code.

Regression checks passed: 37 console/protocol cases and 28 task exchanges per
profile; exception injection per profile; terminal integration including the new
commands; fifteen panel tests (eleven pure host, four real QEMU). Formatting,
kernel/default and exception-feature Clippy, console/panel Clippy, and an optimized
host-panel build passed. No GUI code changed, so native-window review was not
repeated. Captures: [debug](evidence/2026-10-27-memory-debug.txt) and
[release](evidence/2026-10-27-memory-release.txt).

The captures above preserve the earlier, pre-MMU increment with 256 free pages.
Current MMU captures and protection checks are in [MMU.md](MMU.md). The allocator
now reports total=256, used=5, free=251; diagnostic completion preserves those
counts and the live mappings. Isolated services remain pending.

Reference: [QEMU virt bare-metal hardware configuration](https://www.qemu.org/docs/master/system/arm/virt.html#hardware-configuration-information-for-bare-metal-programming).

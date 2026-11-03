# What limits generated applications

Owning the OS source gives us the ability to implement new mechanisms. It does
not make those mechanisms available before they exist. The user can request new
behavior; Meta can describe behavior using the current language; Rust must have
an implementation that can execute and validate it. These are three different
kinds of control. A model proposal never grants itself additional authority.

## Current boundary, November 3

| Layer | Available | Missing / consequence |
| --- | --- | --- |
| App language | Typed forms, integer formulas, tables/totals, checkbox filtering, guarded manual workflows | No general algorithm, custom layout, relations, timers, embedded AI or arbitrary code |
| Data | 3 apps; 6 fields; 8 rows per app; 32 printable ASCII bytes per cell | No deletion, pagination, scalable database, Unicode input or decimal type |
| Process | Up to 4 actions; numeric guards; local per-row state; terminal edit lock | No cross-record transaction, action history, background execution or editable process graph |
| System effects | User-applied software framebuffer brightness | No generated-app camera, GPS, network, notifications, arbitrary file or physical backlight access |
| Durability | Host checkpoint; reviewed CSV import/export | No guest disk driver/database; CSV omits process states |
| Execution | Generic Rust interpreter and native QEMU framebuffer | App runtime/UI still share EL1; existing EL0 worker isolation is not wired into apps |
| Generation | Meta returns a strict schema; user reviews before installation | No native executable compiler, dynamic loader or app syscall ABI |
| Input | 128 ASCII prompt characters; 1 model request at a time, 8 per session | Not an unrestricted long-form app specification; no automatic retry |

The complete catalog is [APP_CAPABILITIES.md](APP_CAPABILITIES.md). A supported
workshop demonstrates a complete small process, not a complete mobile platform.
Increasing a constant is not a sufficient scaling implementation: arrays, copies,
wire sizes, snapshots, CSV staging, screen rows and tests all depend on the bounds.
A prior debug stack failure already demonstrated why this matters; see
[KERNEL_DECISION.md](KERNEL_DECISION.md).

## Engineering sequence after the submission slice

1. **Records that can grow:** stable record IDs, reviewed delete/archive,
   capacity-aware pagination, explicit quotas, and storage outside large stack
   frames. Choose a bounded new capacity using measured memory/stack budgets;
   test full capacity, reuse, invalid IDs, migration and restart before raising
   the catalog limits. Deletion must not silently bypass terminal workflow rules.
2. **Related business data:** references between records (for example orders and
   parts), typed validation, sorting/search and transactional stock changes.
   Specify reference ownership, deleted references and all-or-nothing updates.
   This is the next meaningful step toward a workshop management product.
3. **Richer portable apps:** reusable layouts, Unicode text/fonts and explicit
   numeric types. Broaden the schema and renderer together; keep older saved
   schemas readable. Do not advertise Excel compatibility from an arithmetic table.
4. **Services and events:** connect the app runtime to a real EL0 service boundary,
   capability-checked IPC, cancellable work and execution budgets. Add event
   delivery and scoped storage through that boundary. Test crashes and denied
   cross-app access before claiming process isolation.
5. **Device-backed capabilities:** one driver/service and user permission flow at
   a time. Network, location or camera need actual implementations and device
   support; they cannot be synthesized by giving a model a larger prompt.
6. **Optional native-code apps:** define the SDK/ABI, bounded build environment,
   executable validation/loader and isolated process lifecycle. Native generation
   is a separate app type, not a rename of the existing schema interpreter.

These are ordered design tasks, not promised delivery dates or implemented
features. Some storage/service work must move earlier if new capacity requires it.
A full OS is an ongoing platform; completion should mean a tested product slice.
For this deadline that slice is describe -> review -> install -> use -> evolve
compatibly -> save/recover, within the disclosed runtime. [DEMO_READY.md](DEMO_READY.md)
turns that slice into a runnable demonstration.

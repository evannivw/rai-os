# Why an own kernel, and what it does not prove

RAI-OS has two objectives: learning/building an operating system over the long
term, and demonstrating useful Meta-generated tools during the hackathon. Starting
with an own Rust kernel serves the first objective directly. It is the slower
route for the second: boot, memory, exceptions, UART, graphics and diagnostics
consume time before the user-facing app story becomes useful.

The current app creation and workflow experience could also be implemented on
Linux. An own kernel is not necessary for AI-generated apps, and Linux can also
be customized. Do not present kernel ownership as a unique ability to add AI or
as evidence that fewer layers automatically outperform existing systems.

Our kernel gives direct ownership of the experimental ABI, scheduling, memory
mapping and future service/capability design. Its value must become visible in
measurable behavior: enforceable permissions, controlled resource budgets, useful
isolation and predictable execution. Rust, small code size and kernel ownership
alone prove none of those advantages. The current app runtime is bounded and
checks operations, but still shares EL1 privilege/address space with the kernel.
The host provides HTTPS, input transport and persistent file adapters.

Keep the kernel as the project's systems foundation, while adding app-facing
features to domain/controller/adapter modules above hardware mechanisms. A repair
workflow does not belong in a scheduler or device driver: its schema belongs to
the generated app, and generic transition validation belongs to the app runtime.
Keep admission/fast execution independent of inference. This preserves a path to
real user services and process isolation later without embedding business rules
in low-level code now.

For the current deadline, do not restart on another kernel or add drivers without
a concrete demo need. Demonstrate a useful complete app process and its checked
failure paths, then publish exact limits and reproducible results. Future phone
support requires concrete bootloader/SoC/device work; an AArch64 QEMU guest does
not imply that the OS installs on arbitrary Android phones.

## November 3 implementation lesson

The first debug workflow installation exposed stack pressure. Compiler assembly
showed the retained console loop reserving 55,856 bytes while other handlers still
needed space on the fixed 65,536-byte kernel stack. Separating large console
construction temporaries into a non-inlined initializer reduced that retained
frame to 32,272 bytes with this pinned compiler/profile. The initializer has its
own temporary frame, which returns before command execution. No stack reservation
or MMU window increase was made. Real debug/release execution must still verify
the relevant paths; individual compiler frame sizes are not a complete worst-case
stack proof. This is infrastructure work an own kernel makes our responsibility.


## November 3: Git and safe platform evolution

Git belongs in a service/tool adapter, not in the kernel. The first
[Git integration](APP_GIT.md) versions checked app definitions on the host and
proposes compatible restores through native approval. It requires no new kernel
mechanism and does not authorize changing executable RAI-OS source.

The next systems mechanisms should follow concrete boundaries: move the app
interpreter and controller into isolated user services with capability-checked
IPC; add guest-backed durable storage when removing the host checkpoint becomes
a product requirement; add device services for specific implemented hardware.
Porting Git into the guest would need a suitable filesystem/runtime environment
or a smaller Git-compatible service implementation. A model tool enum does not
supply those mechanisms.

Updating UI/coordinator service code additionally needs candidate build/tests,
resource limits, state migration policy and atomic package activation with a
bootable previous version. Git rollback alone is insufficient. Declarative app
restores already provide a limited example: preserve current data, validate the
old definition against it, require approval and keep every saved version.

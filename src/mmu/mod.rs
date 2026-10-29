//! Immutable base/A/B identity maps with task-private writable user pages.
mod tables;
use crate::memory::{Memory, Page};
use core::{arch::asm, fmt, ptr};
use tables::{Layout, MAIR, TABLE_COUNT, TCR, UserLayout};

#[derive(Clone, Copy)]
pub(crate) enum TaskSpace {
    A,
    B,
}
impl TaskSpace {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }
    pub(crate) fn from_index(index: usize) -> Result<Self, &'static str> {
        match index {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            _ => Err("invalid_task_space"),
        }
    }
}
pub struct Mmu {
    // Retain ownership forever; complete tables are never mutated or released.
    _pages: [Page; TABLE_COUNT],
}
#[derive(PartialEq, Eq)]
pub struct Snapshot {
    sctlr: u64,
    root: u64,
    tcr: u64,
    mair: u64,
}
impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "enabled={}; mode=identity; granule=4096; table_pages={TABLE_COUNT}; root=0x{:016x}; tcr=0x{:016x}; mair=0x{:016x}; wxn={}; caches={}",
            self.sctlr & 1 != 0,
            self.root,
            self.tcr,
            self.mair,
            self.sctlr & (1 << 19) != 0,
            self.sctlr & ((1 << 2) | (1 << 12)) != 0
        )
    }
}
impl Mmu {
    /// # Safety
    /// Exactly once, EL1 on one CPU with interrupts masked and MMU/caches off.
    /// Linker ranges must cover every live instruction, stack, datum and UART;
    /// no firmware/DMA/other owner may modify the pool. The returned table owner
    /// must live for the rest of execution. No table contents change after activation.
    pub unsafe fn enable(memory: &mut Memory) -> Self {
        let prior: u64;
        // SAFETY: privileged EL1 register read under the entry contract.
        unsafe {
            asm!("mrs {value}, sctlr_el1", value = out(reg) prior, options(nomem, nostack, preserves_flags));
        }
        assert_eq!(
            prior & ((1 << 12) | (1 << 2) | 1),
            0,
            "MMU/caches already enabled"
        );
        unsafe extern "C" {
            static __text_start: u8;
            static __text_end: u8;
            static __rodata_end: u8;
            static __user_text_start: u8;
            static __user_text_end: u8;
            static __user_rodata_end: u8;
            static __user_stack_bottom: u8;
            static __user_stack_top: u8;
        }
        let layout = Layout {
            text_start: ptr::addr_of!(__text_start) as usize,
            text_end: ptr::addr_of!(__text_end) as usize,
            rodata_end: ptr::addr_of!(__rodata_end) as usize,
            ram_end: memory.snapshot().end,
            user: UserLayout {
                text_start: ptr::addr_of!(__user_text_start) as usize,
                text_end: ptr::addr_of!(__user_text_end) as usize,
                rodata_end: ptr::addr_of!(__user_rodata_end) as usize,
                stack_start: ptr::addr_of!(__user_stack_bottom) as usize,
                stack_end: ptr::addr_of!(__user_stack_top) as usize,
            },
        };
        let mut pages = core::array::from_fn::<_, TABLE_COUNT, _>(|_| {
            memory.allocate().expect("MMU table allocation failed")
        });
        let addresses = core::array::from_fn(|index| pages[index].physical_address());
        // SAFETY: each owned page is distinct, initialized and writable with MMU
        // off. Borrows end before activation makes table pages read-only.
        let buffers = pages.each_mut().map(|page| unsafe { page.table_mut() });
        tables::build(buffers, addresses, layout).expect("invalid MMU layout");
        let root = addresses[0] as u64;
        // Preserve architectural reset/reserved bits; keep caches off and little
        // endian. Clear UMA so EL0 cannot mask IRQs. WXN adds a blanket execute-never rule for writable mappings.
        let control =
            (prior & !((1 << 25) | (1 << 24) | (1 << 9) | (1 << 12) | (1 << 2))) | 1 | (1 << 19);
        // SAFETY: tables are complete, cover this PC/SP/vectors/UART by identity,
        // and cannot be released. DSB publishes writes; invalidation and ISBs
        // synchronize the translation context. No nomem: mappings/order change.
        // Table walks and normal RAM both use non-cacheable attributes.
        unsafe {
            asm!(
                "dsb sy",
                "msr mair_el1, {mair}",
                "msr tcr_el1, {tcr}",
                "msr ttbr0_el1, {root}",
                "msr ttbr1_el1, xzr",
                "isb",
                "tlbi vmalle1",
                "dsb sy",
                "isb",
                "msr sctlr_el1, {control}",
                "isb",
                mair = in(reg) MAIR, tcr = in(reg) TCR, root = in(reg) root,
                control = in(reg) control, options(nostack, preserves_flags),
            );
        }
        Self { _pages: pages }
    }
    /// Execute a bounded kernel continuation with a fixed task map, then restore
    /// the base map even if the continuation returns Err. Panic is fatal (abort).
    /// # Safety
    /// One EL1 CPU, all DAIF masked, no active user invocation or nested switch.
    /// The continuation may enter EL0 but must return masked to this kernel stack.
    /// All maps preserve kernel/device addresses and memory attributes exactly.
    pub(crate) unsafe fn with_task<T>(
        &self,
        task: TaskSpace,
        action: impl FnOnce() -> T,
    ) -> Result<T, &'static str> {
        let base = self._pages[0].physical_address() as u64;
        if self.snapshot().root != base || !interrupts_masked() {
            return Err("mmu_switch_state");
        }
        let root = self._pages[6 + task.index() * 3].physical_address() as u64;
        // SAFETY: owned immutable root, identical kernel map, masked single CPU.
        unsafe { switch_root(root) };
        let result = action();
        // SAFETY: continuation contract restores masked EL1; same live kernel map.
        unsafe { switch_root(base) };
        if self.snapshot().root != base || !interrupts_masked() {
            return Err("mmu_restore_state");
        }
        Ok(result)
    }
    pub fn snapshot(&self) -> Snapshot {
        let (sctlr, root, tcr, mair);
        // SAFETY: read-only EL1 registers; no memory/stack access or flag changes.
        unsafe {
            asm!("mrs {sctlr}, sctlr_el1", "mrs {root}, ttbr0_el1", "mrs {tcr}, tcr_el1", "mrs {mair}, mair_el1",
                sctlr = out(reg) sctlr, root = out(reg) root, tcr = out(reg) tcr, mair = out(reg) mair,
                options(nomem, nostack, preserves_flags));
        }
        Snapshot {
            sctlr,
            root,
            tcr,
            mair,
        }
    }
}

fn interrupts_masked() -> bool {
    let daif: u64;
    // SAFETY: read-only EL1 state, no memory access.
    unsafe {
        asm!("mrs {value}, daif", value = out(reg) daif, options(nomem, nostack, preserves_flags));
    }
    daif & 0x3c0 == 0x3c0
}
/// # Safety
/// Root is a live, aligned, immutable owned table. Identical kernel mappings,
/// PA/attributes, one masked EL1 CPU, no concurrent walkers on another core.
unsafe fn switch_root(root: u64) {
    // SAFETY: caller's map preserves this code/stack. All roots use ASID 0;
    // invalidate ALL local EL1 translations (including global entries), and
    // complete invalidation before any user entry. No nomem: memory ordering
    // and permissions change. This is not an ASID/SMP optimization.
    unsafe {
        asm!("dsb sy", "msr ttbr0_el1, {root}", "isb", "tlbi vmalle1",
             "dsb sy", "isb", root = in(reg) root, options(nostack, preserves_flags));
    }
}

#[cfg(any(
    feature = "mmu-test-write-text",
    feature = "mmu-test-write-rodata",
    feature = "mmu-test-write-table",
    feature = "mmu-test-execute-data",
    feature = "mmu-test-unmapped"
))]
mod self_test;
#[cfg(any(
    feature = "mmu-test-write-text",
    feature = "mmu-test-write-rodata",
    feature = "mmu-test-write-table",
    feature = "mmu-test-execute-data",
    feature = "mmu-test-unmapped"
))]
pub use self_test::run_fault;

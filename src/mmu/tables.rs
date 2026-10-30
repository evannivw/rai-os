//! Pure fixed-board table construction. No raw pointers or system registers.
#[cfg(feature = "guest-display")]
#[path = "../display/memory.rs"]
pub(crate) mod display_memory;
pub const PAGE_SIZE: usize = 4096;
pub const TABLE_COUNT: usize = 12;
pub const GICD: usize = 0x0800_0000;
pub const GICC: usize = 0x0801_0000;
pub const UART: usize = 0x0900_0000;
pub const RAM_BASE: usize = 0x4000_0000;
pub const WINDOW_END: usize = 0x4020_0000;
pub const MAIR: u64 = 0x4400; // Attr0: Device-nGnRnE; Attr1: Normal non-cacheable.
// 39-bit TTBR0 VA, 4 KiB granules, inner-shareable/non-cacheable table walks,
// 32-bit PA. TTBR1 walks disabled; its size/granule fields still use valid values.
pub const TCR: u64 = 25 | (3 << 12) | (25 << 16) | (1 << 23) | (2 << 30);
const VALID_PAGE: u64 = 3;
const AF: u64 = 1 << 10;
const INNER_SHAREABLE: u64 = 3 << 8;
const NORMAL: u64 = 1 << 2;
const READ_ONLY: u64 = 1 << 7; // AP=10: privileged read-only, no EL0 access.
const PXN: u64 = 1 << 53;
const UXN: u64 = 1 << 54;

#[derive(Clone, Copy)]
pub struct Layout {
    pub text_start: usize,
    pub text_end: usize,
    pub rodata_end: usize,
    pub ram_end: usize,
    pub user: UserLayout,
}
#[derive(Clone, Copy)]
pub struct UserLayout {
    pub text_start: usize,
    pub text_end: usize,
    pub rodata_end: usize,
    pub stack_start: usize,
    pub stack_end: usize,
}
#[derive(Clone, Copy)]
enum Kind {
    Code,
    ReadOnly,
    Data,
    Device,
    UserCode,
    UserReadOnly,
    UserData,
}
fn leaf(address: usize, kind: Kind) -> u64 {
    let common = address as u64 | VALID_PAGE | AF | UXN;
    match kind {
        Kind::UserCode => (common & !UXN) | NORMAL | INNER_SHAREABLE | READ_ONLY | (1 << 6) | PXN,
        Kind::UserReadOnly => common | NORMAL | INNER_SHAREABLE | READ_ONLY | (1 << 6) | PXN,
        Kind::UserData => common | NORMAL | INNER_SHAREABLE | (1 << 6) | PXN,
        Kind::Code => common | NORMAL | INNER_SHAREABLE | READ_ONLY,
        Kind::ReadOnly => common | NORMAL | INNER_SHAREABLE | READ_ONLY | PXN,
        Kind::Data => common | NORMAL | INNER_SHAREABLE | PXN,
        Kind::Device => common | PXN, // Device shareability is architectural.
    }
}

/// Base: L1 root, low L2, UART L3, RAM L2, RAM L3, GIC L3.
/// Each task adds a private root/RAM L2/RAM L3; low device tables are shared.
/// All unmapped entries remain invalid. Only explicit user regions permit EL0.
/// The optional display adds one EL1-only 2 MiB RAM block, never an alias.
pub fn build(
    tables: [&mut [u64; 512]; TABLE_COUNT],
    addresses: [usize; TABLE_COUNT],
    layout: Layout,
) -> Result<(), &'static str> {
    let bounds = [
        layout.text_start,
        layout.text_end,
        layout.rodata_end,
        layout.ram_end,
    ];
    if bounds.iter().any(|value| !value.is_multiple_of(PAGE_SIZE))
        || layout.text_start != 0x4008_0000
        || layout.text_start >= layout.text_end
        || layout.text_end >= layout.rodata_end
        || layout.rodata_end >= layout.ram_end
        || layout.ram_end > WINDOW_END
    {
        return Err("invalid_mmu_layout");
    }
    let user = layout.user;
    if [
        user.text_start,
        user.text_end,
        user.rodata_end,
        user.stack_start,
        user.stack_end,
    ]
    .iter()
    .any(|v| !v.is_multiple_of(PAGE_SIZE))
        || user.text_start < layout.rodata_end
        || user.text_start >= user.text_end
        || user.text_end >= user.rodata_end
        || user.rodata_end.checked_add(PAGE_SIZE) != Some(user.stack_start)
        || user.stack_start.checked_add(2 * PAGE_SIZE) != Some(user.stack_end)
        || user
            .stack_end
            .checked_add(PAGE_SIZE)
            .is_none_or(|end| end > layout.ram_end)
    {
        return Err("invalid_user_layout");
    }
    for (index, address) in addresses.iter().enumerate() {
        if !address.is_multiple_of(PAGE_SIZE)
            || *address < user.stack_end + PAGE_SIZE
            || *address >= layout.ram_end
            || addresses[..index].contains(address)
        {
            return Err("invalid_table_page");
        }
    }
    let [
        root,
        low_l2,
        uart_l3,
        ram_l2,
        ram_l3,
        gic_l3,
        a_root,
        a_l2,
        a_l3,
        b_root,
        b_l2,
        b_l3,
    ] = tables;
    for table in [
        &mut *root,
        &mut *low_l2,
        &mut *uart_l3,
        &mut *ram_l2,
        &mut *ram_l3,
        &mut *gic_l3,
    ] {
        table.fill(0);
    }
    root[0] = addresses[1] as u64 | 3;
    root[1] = addresses[3] as u64 | 3;
    low_l2[(UART >> 21) & 511] = addresses[2] as u64 | 3;
    uart_l3[(UART >> 12) & 511] = leaf(UART, Kind::Device);
    low_l2[(GICD >> 21) & 511] = addresses[5] as u64 | 3;
    gic_l3[0] = leaf(GICD, Kind::Device);
    gic_l3[16] = leaf(GICC, Kind::Device);
    ram_l2[0] = addresses[4] as u64 | 3;
    #[cfg(feature = "guest-display")]
    {
        const _: () = assert!(display_memory::END - display_memory::START == 2 * 1024 * 1024);
        const _: () = assert!(display_memory::START.is_multiple_of(2 * 1024 * 1024));
        uart_l3[(display_memory::FW_CFG >> 12) & 511] = leaf(display_memory::FW_CFG, Kind::Device);
        // L2 block descriptor: same normal non-cacheable, EL1-only RW/NX policy.
        ram_l2[(display_memory::START >> 21) & 511] = leaf(display_memory::START, Kind::Data) & !2;
    }
    for address in (layout.text_start..layout.ram_end).step_by(PAGE_SIZE) {
        if (user.rodata_end..user.stack_start).contains(&address)
            || (user.stack_end..user.stack_end + PAGE_SIZE).contains(&address)
        {
            continue;
        }
        let kind = if (user.text_start..user.text_end).contains(&address) {
            Kind::UserCode
        } else if (user.text_end..user.rodata_end).contains(&address) {
            Kind::UserReadOnly
        } else if (user.stack_start..user.stack_end).contains(&address) {
            Kind::UserData
        } else if address < layout.text_end {
            Kind::Code
        } else if address < layout.rodata_end || addresses.contains(&address) {
            Kind::ReadOnly
        } else {
            Kind::Data
        };
        ram_l3[(address - RAM_BASE) / PAGE_SIZE] = leaf(address, kind);
    }
    // The kernel mapping is identical in every root. Only EL0 access to the
    // peer's stack page is removed; PA, attributes and EL1 permissions stay fixed.
    for (id, task_root, task_l2, task_l3) in [(0, a_root, a_l2, a_l3), (1, b_root, b_l2, b_l3)] {
        task_root.copy_from_slice(root);
        task_l2.copy_from_slice(ram_l2);
        task_l3.copy_from_slice(ram_l3);
        task_root[1] = addresses[7 + id * 3] as u64 | 3;
        task_l2[0] = addresses[8 + id * 3] as u64 | 3;
        let peer = user.stack_start + (id ^ 1) * PAGE_SIZE;
        if !cfg!(feature = "isolation-test-open-peer") {
            task_l3[(peer - RAM_BASE) / PAGE_SIZE] = leaf(peer, Kind::Data);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn layout() -> Layout {
        Layout {
            text_start: 0x40080000,
            text_end: 0x40084000,
            rodata_end: 0x40086000,
            ram_end: 0x401a0000,
            user: UserLayout {
                text_start: 0x40087000,
                text_end: 0x40088000,
                rodata_end: 0x40089000,
                stack_start: 0x4008a000,
                stack_end: 0x4008c000,
            },
        }
    }
    fn addresses() -> [usize; TABLE_COUNT] {
        core::array::from_fn(|i| 0x40090000 + i * PAGE_SIZE)
    }
    #[test]
    fn permissions_and_sparse_identity_mapping() {
        let mut tables = [[0; 512]; TABLE_COUNT];
        build(tables.each_mut(), addresses(), layout()).unwrap();
        let code = tables[4][0x80];
        let rodata = tables[4][0x84];
        let data = tables[4][0x86];
        let table = tables[4][0x90];
        for entry in [
            code,
            rodata,
            data,
            table,
            tables[2][0],
            tables[5][0],
            tables[5][16],
        ] {
            assert_ne!(entry & AF, 0);
            assert_eq!(entry & (1 << 6), 0); // No EL0 access.
            assert_ne!(entry & UXN, 0);
        }
        assert_eq!(code & (READ_ONLY | PXN), READ_ONLY);
        assert_eq!(rodata & (READ_ONLY | PXN), READ_ONLY | PXN);
        assert_eq!(table & (READ_ONLY | PXN), READ_ONLY | PXN);
        assert_eq!(data & (READ_ONLY | PXN), PXN);
        assert_eq!(code & 0xffff_f000, 0x40080000);
        assert_eq!(tables[2][0] & (7 << 2), 0); // Device MAIR slot.
        assert_eq!(tables[4][0x87] & ((3 << 6) | PXN | UXN), (3 << 6) | PXN);
        assert_eq!(
            tables[4][0x88] & ((3 << 6) | PXN | UXN),
            (3 << 6) | PXN | UXN
        );
        assert_eq!(
            tables[4][0x8a] & ((3 << 6) | PXN | UXN),
            (1 << 6) | PXN | UXN
        );
        assert_eq!(tables[4][0x89], 0); // Lower stack guard.
        assert_eq!(tables[4][0x8c], 0); // Upper stack guard.
        assert_eq!(tables[4][0], 0); // DTB/low RAM not mapped.
        assert_eq!(tables[4][0x1a0], 0); // Pool end is exclusive.
        assert_eq!(tables[0].iter().filter(|v| **v != 0).count(), 2);
        assert_eq!(tables[1].iter().filter(|v| **v != 0).count(), 2);
        assert_eq!(tables[5].iter().filter(|v| **v != 0).count(), 2);
        assert_eq!(tables[5][0] & ((7 << 2) | PXN | UXN), PXN | UXN);
        assert_eq!(tables[5][16] & ((7 << 2) | PXN | UXN), PXN | UXN);
        assert_eq!(
            tables[2].iter().filter(|v| **v != 0).count(),
            1 + usize::from(cfg!(feature = "guest-display"))
        );
        assert_eq!(
            tables[3].iter().filter(|v| **v != 0).count(),
            1 + usize::from(cfg!(feature = "guest-display"))
        );
    }
    #[test]
    fn task_roots_share_only_read_only_user_regions_and_protect_all_tables() {
        let mut tables = [[0; 512]; TABLE_COUNT];
        let addresses = addresses();
        let layout = layout();
        build(tables.each_mut(), addresses, layout).unwrap();
        let start = (layout.user.stack_start - RAM_BASE) / PAGE_SIZE;
        for id in 0..2 {
            let root = 6 + id * 3;
            assert_eq!(tables[root][0], tables[0][0]);
            assert_eq!(tables[root][1], addresses[root + 1] as u64 | 3);
            assert_eq!(tables[root + 1][0], addresses[root + 2] as u64 | 3);
            assert!(tables[root][2..].iter().all(|e| *e == 0));
            assert_eq!(tables[root + 1][1..], tables[3][1..]);
            for index in 0..512 {
                let expected = if index == start + (id ^ 1) {
                    tables[4][index] & !(1 << 6)
                } else {
                    tables[4][index]
                };
                assert_eq!(tables[root + 2][index], expected);
                let entry = tables[root + 2][index];
                // Only the selected stack page is EL0 writable, with no aliases.
                if entry & 3 == 3 && entry & (1 << 6) != 0 && entry & READ_ONLY == 0 {
                    assert_eq!(index, start + id);
                    assert_eq!(entry & 0xffff_f000, (RAM_BASE + index * PAGE_SIZE) as u64);
                }
            }
            for address in addresses {
                let entry = tables[root + 2][(address - RAM_BASE) / PAGE_SIZE];
                assert_eq!(entry & ((3 << 6) | PXN | UXN), READ_ONLY | PXN | UXN);
            }
        }
    }
    #[cfg(feature = "guest-display")]
    #[test]
    fn display_is_reserved_privileged_non_executable_and_not_aliased() {
        let mut tables = [[0; 512]; TABLE_COUNT];
        build(tables.each_mut(), addresses(), layout()).unwrap();
        let mmio = tables[2][(display_memory::FW_CFG >> 12) & 511];
        assert_eq!(mmio & (3 | (3 << 6) | (7 << 2) | PXN | UXN), 3 | PXN | UXN);
        assert_eq!(mmio & 0xffff_f000, display_memory::FW_CFG as u64);
        for table in [3, 7, 10] {
            let block = tables[table][8];
            assert_eq!(block & 0xffe0_0000, display_memory::START as u64);
            assert_eq!(
                block & (3 | (3 << 6) | (7 << 2) | PXN | UXN),
                1 | NORMAL | PXN | UXN
            );
            assert_eq!(tables[table][7], 0);
            assert_eq!(tables[table][9], 0);
        }
        assert!(layout().ram_end <= display_memory::START);
    }
    #[test]
    fn rejects_invalid_user_regions_before_mutation() {
        for user in [
            UserLayout {
                text_start: 0x40080000,
                ..layout().user
            },
            UserLayout {
                text_end: 0x40088001,
                ..layout().user
            },
            UserLayout {
                stack_start: 0x40089000,
                ..layout().user
            },
            UserLayout {
                stack_end: usize::MAX - 4095,
                ..layout().user
            },
        ] {
            let mut tables = [[42; 512]; TABLE_COUNT];
            assert!(build(tables.each_mut(), addresses(), Layout { user, ..layout() }).is_err());
            assert!(tables.iter().flatten().all(|v| *v == 42));
        }
    }
    #[test]
    fn rejects_bad_boundaries_and_table_aliases_before_mutating() {
        for bad in [
            Layout {
                text_start: 1,
                ..layout()
            },
            Layout {
                text_end: 0x40080000,
                ..layout()
            },
            Layout {
                ram_end: WINDOW_END + PAGE_SIZE,
                ..layout()
            },
        ] {
            let mut tables = [[42; 512]; TABLE_COUNT];
            assert!(build(tables.each_mut(), addresses(), bad).is_err());
            assert!(tables.iter().flatten().all(|v| *v == 42));
        }
        for address in [
            addresses()[1],
            0x40087000,
            0x4008a000,
            0x40080000,
            0x401a0000,
            0x40090001,
        ] {
            let mut pages = addresses();
            pages[0] = address;
            let mut tables = [[42; 512]; TABLE_COUNT];
            assert!(build(tables.each_mut(), pages, layout()).is_err());
            assert!(tables.iter().flatten().all(|v| *v == 42));
        }
    }
}

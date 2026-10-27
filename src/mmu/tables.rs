//! Pure fixed-board table construction. No raw pointers or system registers.
pub const PAGE_SIZE: usize = 4096;
pub const TABLE_COUNT: usize = 5;
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
}
#[derive(Clone, Copy)]
enum Kind {
    Code,
    ReadOnly,
    Data,
    Device,
}
fn leaf(address: usize, kind: Kind) -> u64 {
    let common = address as u64 | VALID_PAGE | AF | UXN;
    match kind {
        Kind::Code => common | NORMAL | INNER_SHAREABLE | READ_ONLY,
        Kind::ReadOnly => common | NORMAL | INNER_SHAREABLE | READ_ONLY | PXN,
        Kind::Data => common | NORMAL | INNER_SHAREABLE | PXN,
        Kind::Device => common | PXN, // Device shareability is architectural.
    }
}

/// Five pages: L1 root, low L2, UART L3, RAM L2, RAM L3.
/// All unmapped entries remain invalid. No blocks, user mappings, or aliases.
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
    for (index, address) in addresses.iter().enumerate() {
        if !address.is_multiple_of(PAGE_SIZE)
            || *address < layout.rodata_end
            || *address >= layout.ram_end
            || addresses[..index].contains(address)
        {
            return Err("invalid_table_page");
        }
    }
    let [root, low_l2, uart_l3, ram_l2, ram_l3] = tables;
    for table in [
        &mut *root,
        &mut *low_l2,
        &mut *uart_l3,
        &mut *ram_l2,
        &mut *ram_l3,
    ] {
        table.fill(0);
    }
    root[0] = addresses[1] as u64 | 3;
    root[1] = addresses[3] as u64 | 3;
    low_l2[(UART >> 21) & 511] = addresses[2] as u64 | 3;
    uart_l3[(UART >> 12) & 511] = leaf(UART, Kind::Device);
    ram_l2[0] = addresses[4] as u64 | 3;
    for address in (layout.text_start..layout.ram_end).step_by(PAGE_SIZE) {
        let kind = if address < layout.text_end {
            Kind::Code
        } else if address < layout.rodata_end || addresses.contains(&address) {
            Kind::ReadOnly
        } else {
            Kind::Data
        };
        ram_l3[(address - RAM_BASE) / PAGE_SIZE] = leaf(address, kind);
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
        }
    }
    fn addresses() -> [usize; TABLE_COUNT] {
        [0x40090000, 0x40091000, 0x40092000, 0x40093000, 0x40094000]
    }
    #[test]
    fn permissions_and_sparse_identity_mapping() {
        let mut tables = [[0; 512]; TABLE_COUNT];
        build(tables.each_mut(), addresses(), layout()).unwrap();
        let code = tables[4][0x80];
        let rodata = tables[4][0x84];
        let data = tables[4][0x86];
        let table = tables[4][0x90];
        for entry in [code, rodata, data, table, tables[2][0]] {
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
        assert_eq!(tables[4][0], 0); // DTB/low RAM not mapped.
        assert_eq!(tables[4][0x1a0], 0); // Pool end is exclusive.
        assert_eq!(tables[0].iter().filter(|v| **v != 0).count(), 2);
        assert_eq!(tables[1].iter().filter(|v| **v != 0).count(), 1);
        assert_eq!(tables[2].iter().filter(|v| **v != 0).count(), 1);
        assert_eq!(tables[3].iter().filter(|v| **v != 0).count(), 1);
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
        for address in [addresses()[1], 0x40080000, 0x401a0000, 0x40090001] {
            let mut pages = addresses();
            pages[0] = address;
            let mut tables = [[42; 512]; TABLE_COUNT];
            assert!(build(tables.each_mut(), pages, layout()).is_err());
            assert!(tables.iter().flatten().all(|v| *v == 42));
        }
    }
}

//! Pure allowlist for the single resumable EL1h diagnostic site.
pub fn may_resume(esr: u64, elr: u64, spsr: u64, expected_return: u64) -> bool {
    // AArch64 SVC, 32-bit instruction, immediate 0x5242; all other ISS bits zero.
    esr == 0x5600_5242
        && elr == expected_return
        && expected_return != 0
        && expected_return.is_multiple_of(4)
        // Allow NZCV variation only. EL1h and every DAIF mask must be preserved.
        && spsr & !0xf000_0000 == 0x3c5
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_only_the_explicit_service_and_return_state() {
        for flags in [0, 0xa000_0000, 0x5000_0000, 0xf000_0000] {
            assert!(may_resume(
                0x5600_5242,
                0x40081000,
                flags | 0x3c5,
                0x40081000
            ));
        }
        for esr in [
            0x5600_5243,
            0x5400_5242,
            0xf200_5242,
            0x9600_000f,
            0x5601_5242,
        ] {
            assert!(!may_resume(esr, 0x40081000, 0x3c5, 0x40081000));
        }
        for state in [0, 0x3c0, 0x3c4, 0x345, 0x13c5, 0x1_0000_03c5] {
            assert!(!may_resume(0x5600_5242, 0x40081000, state, 0x40081000));
        }
        assert!(!may_resume(0x5600_5242, 0x40081004, 0x3c5, 0x40081000));
        assert!(!may_resume(0x5600_5242, 0, 0x3c5, 0));
        assert!(!may_resume(0x5600_5242, 1, 0x3c5, 1));
    }
}

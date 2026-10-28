//! Bounded syscall input rules; no pointers are dereferenced here.
pub const MAX_OUTPUT: usize = 64;
pub const UNKNOWN_CALL: u64 = 1;
pub const BAD_ADDRESS: u64 = 2;
pub const BAD_LENGTH: u64 = 3;
pub const BAD_TEXT: u64 = 4;
pub const OUTPUT_FULL: u64 = 5;

pub fn readable(pointer: u64, length: u64, ranges: [(u64, u64); 2]) -> Result<usize, u64> {
    if length == 0 || length > MAX_OUTPUT as u64 {
        return Err(BAD_LENGTH);
    }
    let end = pointer.checked_add(length).ok_or(BAD_ADDRESS)?;
    if !ranges
        .iter()
        .any(|(start, limit)| pointer >= *start && end <= *limit)
    {
        return Err(BAD_ADDRESS);
    }
    Ok(length as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_overflow_cross_boundary_kernel_and_oversized_requests() {
        let ranges = [(0x1000, 0x2000), (0x3000, 0x4000)];
        assert_eq!(readable(0x1000, 64, ranges), Ok(64));
        assert_eq!(readable(0x3fff, 1, ranges), Ok(1));
        for pointer in [0, 0xfff, 0x2000, 0x2fff, 0x4000, u64::MAX - 3] {
            assert_eq!(readable(pointer, 8, ranges), Err(BAD_ADDRESS));
        }
        assert_eq!(readable(0x1fff, 2, ranges), Err(BAD_ADDRESS));
        for length in [0, 65, u64::MAX] {
            assert_eq!(readable(0x1000, length, ranges), Err(BAD_LENGTH));
        }
    }
}

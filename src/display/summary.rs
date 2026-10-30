//! Bounded presentation metadata; malformed EL0 text must never panic in EL1.
pub fn token_total(index: &str) -> Option<usize> {
    index.split(',').try_fold(0usize, |sum, token| {
        let (_, value) = token.split_once(':')?;
        let value = value.parse::<usize>().ok()?;
        sum.checked_add(value)
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_malformed_or_overflowing_totals_without_wrapping() {
        assert_eq!(token_total("pump:2,review:1"), Some(3));
        for text in [
            "",
            "pump",
            "pump:no",
            "pump:18446744073709551616",
            "x:18446744073709551615,y:1",
        ] {
            assert_eq!(token_total(text), None);
        }
    }
}

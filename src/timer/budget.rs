//! One fixed guest-counter deadline; independent of hardware access and transport.
pub const USER_BUDGET_MS: u64 = 100;
pub struct Budget {
    start: u64,
    pub deadline: u64,
    frequency: u32,
}
impl Budget {
    pub fn new(start: u64, frequency: u32) -> Result<Self, &'static str> {
        if frequency == 0 {
            return Err("invalid_timer_frequency");
        }
        let ticks = (u64::from(frequency) * USER_BUDGET_MS).div_ceil(1000);
        let deadline = start.checked_add(ticks).ok_or("timer_deadline_overflow")?;
        Ok(Self {
            start,
            deadline,
            frequency,
        })
    }
    pub fn expired(&self, now: u64) -> bool {
        now >= self.deadline
    }
    pub fn elapsed_ms(&self, now: u64) -> u64 {
        (u128::from(now.saturating_sub(self.start)) * 1000 / u128::from(self.frequency))
            .min(u128::from(u64::MAX)) as u64
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absolute_deadline_boundary_rounding_and_overflow() {
        let budget = Budget::new(1000, 62_500_000).unwrap();
        assert!(!budget.expired(budget.deadline - 1));
        assert!(budget.expired(budget.deadline));
        assert!(budget.expired(budget.deadline + 1));
        assert_eq!(budget.elapsed_ms(budget.deadline), 100);
        assert_eq!(budget.elapsed_ms(999), 0);
        let coarse = Budget::new(10, 3).unwrap();
        assert_eq!(coarse.deadline, 11); // Never round a positive budget to zero.
        assert!(Budget::new(u64::MAX, 1).is_err());
        assert!(Budget::new(0, 0).is_err());
    }
}

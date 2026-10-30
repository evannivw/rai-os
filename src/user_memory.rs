//! Exclusive ownership of the fixed user RAM across running AND suspended tasks.
use core::sync::atomic::{AtomicBool, Ordering};
static OWNED: AtomicBool = AtomicBool::new(false);
/// Not clonable: retain this token until every saved frame using user RAM is retired.
pub struct Lease(());
impl Lease {
    pub fn acquire() -> Result<Self, &'static str> {
        OWNED
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| Self(()))
            .map_err(|_| "user_memory_busy")
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        OWNED.store(false, Ordering::Release);
    }
}

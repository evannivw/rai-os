//! Read-only Arm generic counter adapter for the supported QEMU EL1 boot.
use crate::command::Uptime;
use core::arch::asm;
use core::num::NonZeroU32;

pub struct Clock {
    started_at: u64,
    frequency: NonZeroU32,
}

pub(crate) fn counter() -> u64 {
    let value;
    // SAFETY: QEMU virt boots us at EL1 with physical counter access enabled.
    // ISB orders the counter read after prior instructions. These instructions
    // do not access the stack or change flags. Do not mark this read pure: the
    // counter advances independently. Other boot environments may trap here.
    unsafe {
        asm!("isb", "mrs {value}, cntpct_el0", value = out(reg) value,
            options(nostack, preserves_flags));
    }
    value
}

impl Clock {
    pub fn start() -> Option<Self> {
        let frequency: u64;
        // SAFETY: CNTFRQ_EL0 is readable at EL1 on the supported machine.
        // QEMU supplies the frequency; this code does not program any timer.
        // Reading the register touches no memory, stack, or condition flags.
        unsafe {
            asm!("mrs {value}, cntfrq_el0", value = out(reg) frequency,
                options(nomem, nostack, preserves_flags));
        }
        // CNTFRQ's architectural frequency field is 32 bits. Reject an invalid
        // value instead of dividing by zero or silently truncating it.
        let frequency = NonZeroU32::new(u32::try_from(frequency).ok()?)?;
        Some(Self {
            started_at: counter(),
            frequency,
        })
    }

    pub fn snapshot(&self) -> Uptime {
        // Supported sessions must not span a counter wrap/reset. Saturating
        // subtraction avoids underflow but does not support live migration or
        // counter reprogramming. QEMU's counter time is not host benchmark time.
        let ticks = counter().saturating_sub(self.started_at);
        // Widen before multiplying, then saturate the public millisecond value.
        // No heap allocation and no floating-point/SIMD instructions are needed.
        let milliseconds = u128::from(ticks) * 1000 / u128::from(self.frequency.get());
        Uptime {
            milliseconds: milliseconds.min(u128::from(u64::MAX)) as u64,
            counter_hz: self.frequency.get(),
        }
    }
}

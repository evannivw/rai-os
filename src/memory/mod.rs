//! Exclusive physical-page pool for our fixed QEMU board. Not a Rust heap or MMU.
mod frames;
use core::{fmt, ptr};
use frames::{Frames, MAX_PAGES, PAGE_SIZE};

// A non-Copy, non-Clone ownership token; only this module can construct one.
// Dropping it without release leaks a page rather than permitting aliasing.
pub struct Page {
    address: usize,
}

pub struct Memory {
    frames: Frames,
    start: usize,
    end: usize,
}
#[derive(Clone, Copy)]
pub struct Snapshot {
    pub start: usize,
    pub end: usize,
    pub total: usize,
    pub free: usize,
}
impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "page_size={PAGE_SIZE}; total_pages={}; free_pages={}; used_pages={}; pool_start=0x{:016x}; pool_end=0x{:016x}",
            self.total,
            self.free,
            self.total - self.free,
            self.start,
            self.end
        )
    }
}
pub struct Check {
    pages: usize,
}
impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "checked_pages={}; exhausted=true; zeroed=true; reused=true; free_pages={}",
            self.pages, self.pages
        )
    }
}
impl Memory {
    /// # Safety
    /// Call exactly once per boot. The linker pool must be exclusively owned,
    /// mapped writable ordinary RAM, on our single-CPU QEMU board. Create before
    /// MMU activation; later mappings must preserve identity for owned pages.
    /// No firmware, DMA, stacks, or other allocator may use these addresses.
    pub unsafe fn from_linker() -> Self {
        unsafe extern "C" {
            static mut __page_pool_start: [u8; PAGE_SIZE * MAX_PAGES];
            static __page_pool_end: u8;
        }
        // Taking symbol addresses does not read the uninitialized NOLOAD region.
        let start = ptr::addr_of!(__page_pool_start) as usize;
        let end = ptr::addr_of!(__page_pool_end) as usize;
        Self {
            frames: Frames::new(start, end).expect("invalid linker page pool"),
            start,
            end,
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            start: self.start,
            end: self.end,
            total: self.frames.total(),
            free: self.frames.free(),
        }
    }
    pub fn allocate(&mut self) -> Option<Page> {
        let address = self.frames.allocate()?;
        // SAFETY: the unique pool owns this aligned page of ordinary RAM. The
        // bitmap marked it used before access, and no Page token currently owns
        // it. The entire page is initialized before the new token is returned.
        unsafe { ptr::write_bytes(address as *mut u8, 0, PAGE_SIZE) };
        Some(Page { address })
    }
    pub fn release(&mut self, page: Page) {
        // Consumes the non-copy token. The unique pool constructor guarantees
        // tokens cannot come from a competing allocator. No raw free is exposed.
        self.frames
            .release(page.address)
            .expect("invalid owned page");
    }
    /// Bounded diagnostic: uses only owned pages, then returns all of them.
    /// An invariant failure panics and halts; it is never reported as success.
    pub fn check(&mut self) -> Result<Check, &'static str> {
        if self.frames.free() == 0 {
            return Err("memory_busy");
        }
        let total = self.frames.free();
        let mut first_address = None;
        let mut pages: [Option<Page>; MAX_PAGES] = [const { None }; MAX_PAGES];
        for (index, slot) in pages[..total].iter_mut().enumerate() {
            let mut page = self.allocate().expect("page allocation failed");
            first_address.get_or_insert(page.address);
            assert!(page.is_zeroed(), "new page is not zeroed");
            page.write_marker(index as u64 + 1);
            *slot = Some(page);
        }
        assert!(self.allocate().is_none(), "pool must be exhausted");
        assert_eq!(self.frames.free(), 0);
        // Verify after ALL allocations, catching writes aliased onto another page.
        for (index, slot) in pages[..total].iter().enumerate() {
            assert!(
                slot.as_ref().unwrap().has_marker(index as u64 + 1),
                "page contents changed"
            );
        }
        for slot in &mut pages[..total] {
            self.release(slot.take().unwrap());
        }
        let reused = self.allocate().expect("released page unavailable");
        assert_eq!(Some(reused.address), first_address);
        assert!(reused.is_zeroed(), "reused page exposes old bytes");
        self.release(reused);
        assert_eq!(self.frames.free(), total);
        Ok(Check { pages: total })
    }
}
impl Page {
    pub(crate) fn physical_address(&self) -> usize {
        self.address
    }
    /// # Safety
    /// Page must be writable and identity mapped (or MMU off), exclusively owned,
    /// and not yet installed in live tables. End this borrow before enabling RO.
    pub(crate) unsafe fn table_mut(&mut self) -> &mut [u64; 512] {
        // SAFETY: caller guarantees the mapping; allocation initializes the full
        // aligned 4 KiB page, and &mut self prevents simultaneous safe borrows.
        unsafe { &mut *(self.address as *mut [u64; 512]) }
    }
    fn is_zeroed(&self) -> bool {
        // SAFETY: a live token exclusively owns this initialized 4 KiB page.
        // Volatile reads make the guest diagnostic observe actual RAM bytes.
        (0..PAGE_SIZE)
            .all(|offset| unsafe { ptr::read_volatile((self.address + offset) as *const u8) == 0 })
    }
    fn write_marker(&mut self, marker: u64) {
        // SAFETY: both u64 locations are aligned and inside this exclusively
        // owned page. No references to its contents exist or escape this module.
        unsafe {
            ptr::write_volatile(self.address as *mut u64, marker);
            ptr::write_volatile((self.address + PAGE_SIZE - 8) as *mut u64, !marker);
        }
    }
    fn has_marker(&self, marker: u64) -> bool {
        // SAFETY: same owned, aligned, initialized locations as write_marker.
        unsafe {
            ptr::read_volatile(self.address as *const u64) == marker
                && ptr::read_volatile((self.address + PAGE_SIZE - 8) as *const u64) == !marker
        }
    }
}

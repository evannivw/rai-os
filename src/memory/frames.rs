//! Address bookkeeping only: no pointers, hardware access, or allocation.
pub const PAGE_SIZE: usize = 4096;
pub const MAX_PAGES: usize = 256;

pub struct Frames {
    start: usize,
    pages: usize,
    used: [u64; MAX_PAGES / 64],
}
impl Frames {
    pub fn new(start: usize, end: usize) -> Result<Self, &'static str> {
        let size = end.checked_sub(start).ok_or("invalid_memory_range")?;
        if !start.is_multiple_of(PAGE_SIZE)
            || !end.is_multiple_of(PAGE_SIZE)
            || size == 0
            || size / PAGE_SIZE > MAX_PAGES
        {
            return Err("invalid_memory_range");
        }
        Ok(Self {
            start,
            pages: size / PAGE_SIZE,
            used: [0; MAX_PAGES / 64],
        })
    }
    pub fn total(&self) -> usize {
        self.pages
    }
    pub fn free(&self) -> usize {
        self.pages
            - self
                .used
                .iter()
                .map(|word| word.count_ones() as usize)
                .sum::<usize>()
    }
    pub fn allocate(&mut self) -> Option<usize> {
        for page in 0..self.pages {
            let mask = 1u64 << (page % 64);
            let word = &mut self.used[page / 64];
            if *word & mask == 0 {
                *word |= mask;
                return Some(self.start + page * PAGE_SIZE);
            }
        }
        None
    }
    pub fn release(&mut self, address: usize) -> Result<(), &'static str> {
        let offset = address.checked_sub(self.start).ok_or("invalid_page")?;
        if !offset.is_multiple_of(PAGE_SIZE) || offset / PAGE_SIZE >= self.pages {
            return Err("invalid_page");
        }
        let page = offset / PAGE_SIZE;
        let mask = 1u64 << (page % 64);
        let word = &mut self.used[page / 64];
        if *word & mask == 0 {
            return Err("page_already_free");
        }
        *word &= !mask;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhaustion_uniqueness_and_reuse_cross_bitmap_words() {
        let start = 0x41000000;
        let mut frames = Frames::new(start, start + MAX_PAGES * PAGE_SIZE).unwrap();
        for page in 0..MAX_PAGES {
            assert_eq!(frames.allocate(), Some(start + page * PAGE_SIZE));
            assert_eq!(frames.free(), MAX_PAGES - page - 1);
        }
        assert_eq!(frames.allocate(), None);
        for page in [0, 63, 64, 127, 128, 255] {
            frames.release(start + page * PAGE_SIZE).unwrap();
        }
        for page in [0, 63, 64, 127, 128, 255] {
            assert_eq!(frames.allocate(), Some(start + page * PAGE_SIZE));
        }
        assert_eq!(frames.allocate(), None);
        for page in 0..MAX_PAGES {
            frames.release(start + page * PAGE_SIZE).unwrap();
        }
        assert_eq!(frames.free(), MAX_PAGES);
    }
    #[test]
    fn rejects_invalid_frees_without_changing_ownership() {
        let mut frames = Frames::new(0x1000, 0x3000).unwrap();
        let page = frames.allocate().unwrap();
        for address in [0, 0xfff, 0x1001, 0x3000, usize::MAX] {
            assert_eq!(frames.release(address), Err("invalid_page"));
            assert_eq!(frames.free(), 1);
        }
        assert_eq!(frames.release(0x2000), Err("page_already_free"));
        frames.release(page).unwrap();
        assert_eq!(frames.release(page), Err("page_already_free"));
        assert_eq!(frames.free(), 2);
    }
    #[test]
    fn validates_ranges_and_partial_bitmap_capacity() {
        for (start, end) in [
            (0, 0),
            (4096, 0),
            (1, 4096),
            (0, 4097),
            (0, (MAX_PAGES + 1) * PAGE_SIZE),
            (0, usize::MAX),
        ] {
            assert!(Frames::new(start, end).is_err());
        }
        let mut frames = Frames::new(4096, 4096 * 66).unwrap();
        for _ in 0..65 {
            assert!(frames.allocate().is_some());
        }
        assert_eq!(frames.total(), 65);
        assert_eq!(frames.allocate(), None);
    }
}

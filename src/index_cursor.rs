//! Constant-space EL0 serializer shared by standalone and retained index workers.
//! Returns the next ASCII byte, or zero at end; no syscall or transport dependency.
pub struct Cursor {
    id: u64,
    previous: u64,
    token: u64,
    position: u64,
    phase: u64,
}
impl Cursor {
    #[inline(always)]
    pub fn new(id: u64) -> Self {
        Self {
            id,
            previous: 0,
            token: 0,
            position: 0,
            phase: 0,
        }
    }
    // SAFETY: EL0 text; cursor references belong to the calling program's private
    // stack. Only bounded immutable user records are read; no kernel-only helpers.
    #[unsafe(link_section = ".user.text")]
    #[inline(never)]
    pub fn next(&mut self) -> u64 {
        loop {
            if self.phase == 0 {
                self.token = crate::records::next_token(self.id, self.previous);
                if self.token == 0 {
                    self.phase = 4;
                    return if self.previous == 0 { b'-' as u64 } else { 0 };
                }
                self.position = 0;
                self.phase = 1;
                if self.previous != 0 {
                    return b',' as u64;
                }
            } else if self.phase == 1 {
                if self.position < ((self.token >> 8) & 255) {
                    let offset = (self.token & 255) - 1 + self.position;
                    self.position += 1;
                    return crate::records::token_byte(self.id, offset as usize) as u64;
                }
                self.phase = 2;
                return b':' as u64;
            } else if self.phase == 2 {
                self.phase = 3;
                let count = (self.token >> 16) & 255;
                if count >= 10 {
                    return b'0' as u64 + count / 10;
                }
            } else if self.phase == 3 {
                self.phase = 0;
                self.previous = self.token & 255;
                return b'0' as u64 + ((self.token >> 16) & 255) % 10;
            } else {
                return 0;
            }
        }
    }
}

//! First exact per-record index producer. Formatting lives outside token selection.
use super::program::{call, exit};
use crate::records::{next_token, token_byte};
#[inline(always)]
fn emit(byte: u8) {
    // One stack byte stays live across the synchronous checked copy syscall.
    let (written, error) = call(1, core::ptr::addr_of!(byte) as u64, 1);
    if written != 1 || error != 0 {
        exit(1);
    }
}
// SAFETY: fixed EL0 entry, exclusive mapped user stack, validated scalar RecordId.
// Only SVC write/exit crosses privilege; no device or kernel pointer access.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".user.text")]
pub extern "C" fn record_index(id: u64) -> ! {
    if cfg!(feature = "index-test-partial") {
        emit(b'x');
        exit(1);
    }
    if cfg!(feature = "index-test-spin") {
        loop {
            core::hint::spin_loop();
        }
    }
    let mut previous = 0;
    loop {
        let token = next_token(id, previous);
        if token == 0 {
            if previous == 0 {
                emit(b'-');
            }
            exit(0);
        }
        if previous != 0 {
            emit(b',');
        }
        let offset = (token & 255) - 1;
        let length = (token >> 8) & 255;
        let count = (token >> 16) & 255;
        let mut i = 0;
        while i < length {
            emit(token_byte(id, (offset + i) as usize));
            i += 1;
        }
        emit(b':');
        // At most 64 tokens fit in a 128-byte source, so two decimal digits suffice.
        if count >= 10 {
            emit(b'0' + (count / 10) as u8);
        }
        emit(b'0' + (count % 10) as u8);
        previous = offset + 1;
    }
}

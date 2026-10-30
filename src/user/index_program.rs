//! First exact per-record index producer. Formatting lives outside token selection.
use super::program::{call, exit};
use crate::index_cursor::Cursor;
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
    let mut cursor = Cursor::new(id);
    loop {
        let byte = cursor.next();
        if byte == 0 {
            exit(0);
        }
        emit(byte as u8);
    }
}

//! Immutable public fixture records and a bounded ASCII token-count operation.
//! No UART, scheduling or provider dependency. Denied source bytes are not linked.
use core::fmt;
pub const MAX_BYTES: usize = 128;
#[derive(Clone, Copy)]
pub struct RecordId(u64);
impl RecordId {
    pub fn from_raw(raw: u64) -> Result<Self, &'static str> {
        if (1..=6).contains(&raw) {
            Ok(Self(raw))
        } else {
            Err("unknown_record")
        }
    }
    pub fn parse(key: &str) -> Result<Self, &'static str> {
        let id = match key {
            "brief@1" => 1,
            "notes@1" => 2,
            "notes@2" => 3,
            "checklist@1" => 4,
            "empty@1" => 5,
            "appendix@1" => 6,
            "restricted@1" => return Err("record_denied"),
            "" => return Err("bad_args"),
            _ if key.bytes().any(|b| !b.is_ascii_graphic()) => return Err("bad_args"),
            _ => return Err("unknown_record"),
        };
        Ok(Self(id))
    }
    pub fn raw(self) -> u64 {
        self.0
    }
    pub fn bytes(self) -> u64 {
        RECORDS[self.0 as usize - 1].length as u64
    }
}
impl fmt::Display for RecordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            [
                "brief@1",
                "notes@1",
                "notes@2",
                "checklist@1",
                "empty@1",
                "appendix@1",
            ][self.0 as usize - 1],
        )
    }
}
struct RecordData {
    bytes: [u8; MAX_BYTES],
    length: usize,
}
const fn record(bytes: &[u8]) -> RecordData {
    assert!(bytes.len() <= MAX_BYTES);
    let mut result = RecordData {
        bytes: [0; MAX_BYTES],
        length: bytes.len(),
    };
    let mut i = 0;
    while i < bytes.len() {
        result.bytes[i] = bytes[i];
        i += 1;
    }
    result
}
// SAFETY: immutable bytes only, deliberately shared read-only with both EL0 tasks.
// Embedded records are public synthetic data; this is not per-record confidentiality.
#[unsafe(link_section = ".user.rodata")]
static RECORDS: [RecordData; 6] = [
    record(include_bytes!(
        "../fixtures/review_pack/v1/records/brief@1.txt"
    )),
    record(include_bytes!(
        "../fixtures/review_pack/v1/records/notes@1.txt"
    )),
    record(include_bytes!(
        "../fixtures/review_pack/v1/records/notes@2.txt"
    )),
    record(include_bytes!(
        "../fixtures/review_pack/v1/records/checklist@1.txt"
    )),
    record(include_bytes!(
        "../fixtures/review_pack/v1/records/empty@1.txt"
    )),
    record(include_bytes!(
        "../fixtures/review_pack/v1/records/appendix@1.txt"
    )),
];
/// Maximal runs of ASCII A-Z/a-z/0-9. Punctuation and whitespace separate tokens.
/// Zero denotes an invalid record ID; valid packed results always include an ID.
/// Layout: ID in bits 32..63, scanned bytes in 16..31, token count in 0..15.
// SAFETY: callable with scalar input, linked into the EL0 executable region.
// Reads only RECORDS, which is bounded, immutable and mapped user read-only.
#[unsafe(link_section = ".user.text")]
#[inline(never)]
#[allow(clippy::manual_is_ascii_check)] // core helper lives in EL1 text in debug builds.
pub fn count(id: u64) -> u64 {
    if id == 0 || id > 6 {
        return 0;
    }
    let source = &RECORDS[id as usize - 1];
    let mut i = 0;
    let mut tokens = 0_u64;
    let mut in_token = false;
    while i < source.length {
        let byte = source.bytes[i];
        let member = matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9');
        if member && !in_token {
            tokens += 1;
        }
        in_token = member;
        i += 1;
    }
    (id << 32) | ((i as u64) << 16) | tokens
}

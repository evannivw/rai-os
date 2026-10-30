//! Bounded index wire format. No task, pointer, UART, model or source-file access.
pub const CAPACITY: usize = 384;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Text {
    bytes: [u8; CAPACITY],
    length: usize,
}
impl Text {
    pub const EMPTY: Self = Self {
        bytes: [0; CAPACITY],
        length: 0,
    };
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.length]).unwrap_or("")
    }
    pub fn len(&self) -> usize {
        self.length
    }
}
pub struct Stream {
    text: Text,
    chunks: u8,
    padded: bool,
}
impl Stream {
    pub fn new() -> Self {
        Self {
            text: Text::EMPTY,
            chunks: 0,
            padded: false,
        }
    }
    pub fn len(&self) -> usize {
        self.text.len()
    }
    pub fn push(&mut self, word: u64) -> Result<(), &'static str> {
        if self.padded || word >> 56 != u64::from(self.chunks) {
            return Err("coord_index_order");
        }
        let start = self.text.length;
        for shift in (0..56).step_by(8) {
            let byte = ((word >> shift) & 255) as u8;
            if byte == 0 {
                self.padded = true;
                continue;
            }
            if self.padded || !matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b',' | b':' | b'-') {
                return Err("coord_index_encoding");
            }
            if self.text.length == CAPACITY {
                return Err("coord_index_limit");
            }
            self.text.bytes[self.text.length] = byte;
            self.text.length += 1;
        }
        if start == self.text.length {
            return Err("coord_index_encoding");
        }
        self.chunks += 1;
        Ok(())
    }
    pub fn finish(self, length: u64, tokens: u64) -> Result<Text, &'static str> {
        if self.text.length == 0 || length != self.text.length as u64 {
            return Err("coord_index_length");
        }
        let text = self.text.as_str();
        if text == "-" {
            return if tokens == 0 {
                Ok(self.text)
            } else {
                Err("coord_index_format")
            };
        }
        let mut previous = "";
        let mut total = 0_u64;
        for entry in text.split(',') {
            let (word, count) = entry.split_once(':').ok_or("coord_index_format")?;
            if word.is_empty()
                || word.len() > 128
                || word <= previous
                || !word
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                || count.starts_with('0')
                || !count.bytes().all(|b| b.is_ascii_digit())
            {
                return Err("coord_index_format");
            }
            let count = count
                .parse::<u64>()
                .ok()
                .filter(|n| (1..=64).contains(n))
                .ok_or("coord_index_format")?;
            total = total.checked_add(count).ok_or("coord_index_format")?;
            previous = word;
        }
        if total != tokens {
            return Err("coord_index_format");
        }
        Ok(self.text)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn stream(text: &str) -> Stream {
        let mut stream = Stream::new();
        for (n, chunk) in text.as_bytes().chunks(7).enumerate() {
            let mut word = (n as u64) << 56;
            for (i, byte) in chunk.iter().enumerate() {
                word |= u64::from(*byte) << (i * 8);
            }
            stream.push(word).unwrap();
        }
        stream
    }
    #[test]
    fn exact_sequence_padding_length_and_capacity_are_required() {
        assert_eq!(
            stream("pump:2,review:1").finish(15, 3).unwrap().as_str(),
            "pump:2,review:1"
        );
        assert!(stream("pump:2").finish(7, 2).is_err());
        assert!(Stream::new().push((1 << 56) | b'a' as u64).is_err());
        assert!(Stream::new().push(0).is_err());
        for byte in [b'A', b' ', b'/', 255] {
            assert!(Stream::new().push(u64::from(byte)).is_err());
        }
        let full = format!(
            "{}:1,{}:1,{}:1",
            "a".repeat(128),
            "b".repeat(128),
            "c".repeat(120)
        );
        assert_eq!(full.len(), CAPACITY);
        assert_eq!(
            stream(&full).finish(CAPACITY as u64, 3).unwrap().as_str(),
            full
        );
        assert!(
            Stream::new()
                .push((b'b' as u64) << 16 | b'a' as u64)
                .is_err()
        );
        let mut s = stream("a:1");
        assert!(s.push((1 << 56) | b'b' as u64).is_err());
        let mut s = stream(&"a".repeat(378));
        assert_eq!(
            s.push((54 << 56) | 0x0061616161616161),
            Err("coord_index_limit")
        );
    }
    #[test]
    fn format_is_canonical_but_does_not_claim_source_correctness() {
        assert_eq!(stream("-").finish(1, 0).unwrap().as_str(), "-");
        for text in [
            "-", "a:0", "a:01", "a:65", "a:1,a:1", "b:1,a:1", "a:1,", "a::1", "a:1,b:2",
        ] {
            assert!(stream(text).finish(text.len() as u64, 2).is_err(), "{text}");
        }
        assert!(stream("wrong:2").finish(7, 2).is_ok()); // Exact content needs the independent checker.
    }
}

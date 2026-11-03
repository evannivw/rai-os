//! Bounded volatile file store. Pure Rust; no MMIO, allocation or host filesystem.
use core::fmt;
pub const FILES: usize = 8;
pub const BYTES: usize = 512;
const PATH: usize = 48;
#[derive(Clone, Copy)]
pub struct Name {
    data: [u8; PATH],
    len: usize,
}
impl Name {
    const EMPTY: Self = Self {
        data: [0; PATH],
        len: 0,
    };
    fn parse(s: &str) -> Result<Self, &'static str> {
        if !s.starts_with('/')
            || s.len() > PATH
            || s.len() < 2
            || !s
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b))
            || s[1..]
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == "..")
        {
            return Err("fs_bad_path");
        }
        let mut n = Self::EMPTY;
        n.data[..s.len()].copy_from_slice(s.as_bytes());
        n.len = s.len();
        Ok(n)
    }
    fn text(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}
#[derive(Clone, Copy)]
pub struct File {
    name: Name,
    data: [u8; BYTES],
    len: usize,
}
impl File {
    const EMPTY: Self = Self {
        name: Name::EMPTY,
        data: [0; BYTES],
        len: 0,
    };
}
struct Stage {
    file: File,
    written: usize,
    generation: u32,
    ticket: u32,
}
pub struct Store {
    files: [File; FILES],
    generation: u32,
    ticket: u32,
    stage: Option<Stage>,
}
pub enum Operation<'a> {
    List,
    Read(&'a str),
    Begin(&'a str),
    Append(&'a str),
    Commit(&'a str),
    Abort(&'a str),
    Move(&'a str),
    Copy(&'a str),
}
impl<'a> Operation<'a> {
    pub fn parse(name: &str, arg: Option<&'a str>) -> Result<Self, &'static str> {
        if name == "fs.list" {
            return if arg.is_none() {
                Ok(Self::List)
            } else {
                Err("bad_args")
            };
        }
        let arg = arg.ok_or("bad_args")?;
        Ok(match name {
            "fs.read" => Self::Read(arg),
            "fs.begin" => Self::Begin(arg),
            "fs.append" => Self::Append(arg),
            "fs.commit" => Self::Commit(arg),
            "fs.abort" => Self::Abort(arg),
            "fs.move" => Self::Move(arg),
            "fs.copy" => Self::Copy(arg),
            _ => return Err("unknown_operation"),
        })
    }
}
pub enum Reply {
    List {
        generation: u32,
        names: [Name; FILES],
        sizes: [usize; FILES],
        staged: bool,
    },
    Data {
        generation: u32,
        file: File,
    },
    State {
        generation: u32,
        ticket: u32,
        written: usize,
    },
}
impl fmt::Display for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State {
                generation,
                ticket,
                written,
            } => write!(
                f,
                "generation={generation}; ticket={ticket}; written={written}"
            ),
            Self::Data { generation, file } => {
                write!(
                    f,
                    "generation={generation}; path={}; bytes={}; data=",
                    file.name.text(),
                    file.len
                )?;
                for b in &file.data[..file.len] {
                    write!(f, "{b:02x}")?;
                }
                Ok(())
            }
            Self::List {
                generation,
                names,
                sizes,
                staged,
            } => {
                write!(
                    f,
                    "generation={generation}; capacity={FILES}; max_bytes={BYTES}; persistence=ram; staged={staged}; files="
                )?;
                let mut first = true;
                for (name, size) in names.iter().zip(sizes) {
                    if name.len > 0 {
                        if !first {
                            f.write_str(",")?;
                        }
                        write!(f, "{}:{size}", name.text())?;
                        first = false;
                    }
                }
                if first {
                    f.write_str("-")?;
                }
                Ok(())
            }
        }
    }
}
fn number(s: &str) -> Result<u32, &'static str> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err("fs_bad_number");
    }
    s.parse().map_err(|_| "fs_bad_number")
}
fn triple(s: &str) -> Result<(&str, &str, &str), &'static str> {
    let mut it = s.split(':');
    let a = it.next().ok_or("bad_args")?;
    let b = it.next().ok_or("bad_args")?;
    let c = it.next().ok_or("bad_args")?;
    if it.next().is_some() {
        return Err("bad_args");
    }
    Ok((a, b, c))
}
impl Store {
    pub const fn new() -> Self {
        Self {
            files: [File::EMPTY; FILES],
            generation: 1,
            ticket: 0,
            stage: None,
        }
    }
    fn locate(&self, name: &Name) -> Option<usize> {
        self.files
            .iter()
            .position(|f| f.name.len > 0 && f.name.text() == name.text())
    }
    fn free(&self) -> Result<usize, &'static str> {
        self.files
            .iter()
            .position(|f| f.name.len == 0)
            .ok_or("fs_full")
    }
    fn next(&self, generation: u32) -> Result<u32, &'static str> {
        if generation != self.generation {
            return Err("fs_stale_generation");
        }
        self.generation
            .checked_add(1)
            .ok_or("fs_generation_exhausted")
    }
    fn state(&self) -> Reply {
        Reply::State {
            generation: self.generation,
            ticket: self.stage.as_ref().map_or(0, |s| s.ticket),
            written: self.stage.as_ref().map_or(0, |s| s.written),
        }
    }
    #[cfg(feature = "guest-display")]
    pub fn entries(&self) -> impl Iterator<Item = (&str, usize)> {
        self.files
            .iter()
            .filter(|f| f.name.len > 0)
            .map(|f| (f.name.text(), f.len))
    }
    #[cfg(feature = "guest-display")]
    pub fn generation(&self) -> u32 {
        self.generation
    }
    pub fn execute(&mut self, op: Operation<'_>) -> Result<Reply, &'static str> {
        match op {
            Operation::List => {
                let mut names = [Name::EMPTY; FILES];
                let mut sizes = [0; FILES];
                for (i, file) in self.files.iter().enumerate() {
                    names[i] = file.name;
                    sizes[i] = file.len;
                }
                return Ok(Reply::List {
                    generation: self.generation,
                    names,
                    sizes,
                    staged: self.stage.is_some(),
                });
            }
            Operation::Read(path) => {
                let name = Name::parse(path)?;
                let i = self.locate(&name).ok_or("fs_not_found")?;
                return Ok(Reply::Data {
                    generation: self.generation,
                    file: self.files[i],
                });
            }
            Operation::Begin(arg) => {
                let (generation, size, path) = triple(arg)?;
                let generation = number(generation)?;
                self.next(generation)?;
                let len = number(size)? as usize;
                let name = Name::parse(path)?;
                if len > BYTES {
                    return Err("fs_file_too_large");
                }
                if self.stage.is_some() {
                    return Err("fs_upload_busy");
                }
                if self.locate(&name).is_some() {
                    return Err("fs_exists");
                }
                self.free()?;
                let ticket = self.ticket.checked_add(1).ok_or("fs_ticket_exhausted")?;
                self.stage = Some(Stage {
                    file: File {
                        name,
                        len,
                        ..File::EMPTY
                    },
                    written: 0,
                    generation,
                    ticket,
                });
                self.ticket = ticket;
            }
            Operation::Append(arg) => {
                let (ticket, offset, hex) = triple(arg)?;
                let ticket = number(ticket)?;
                let offset = number(offset)? as usize;
                if hex.is_empty() || hex.len() > 128 || !hex.len().is_multiple_of(2) {
                    return Err("fs_bad_hex");
                }
                let mut bytes = [0u8; 64];
                for (i, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
                    let digit = |b: u8| match b {
                        b'0'..=b'9' => Some(b - b'0'),
                        b'a'..=b'f' => Some(b - b'a' + 10),
                        b'A'..=b'F' => Some(b - b'A' + 10),
                        _ => None,
                    };
                    bytes[i] = digit(pair[0]).ok_or("fs_bad_hex")? * 16
                        + digit(pair[1]).ok_or("fs_bad_hex")?;
                }
                let stage = self.stage.as_mut().ok_or("fs_no_upload")?;
                if ticket != stage.ticket {
                    return Err("fs_stale_ticket");
                }
                if offset != stage.written {
                    return Err("fs_bad_offset");
                }
                let end = offset
                    .checked_add(hex.len() / 2)
                    .filter(|n| *n <= stage.file.len)
                    .ok_or("fs_upload_overflow")?;
                stage.file.data[offset..end].copy_from_slice(&bytes[..hex.len() / 2]);
                stage.written = end;
            }
            Operation::Commit(arg) => {
                let ticket = number(arg)?;
                let stage = self.stage.as_ref().ok_or("fs_no_upload")?;
                if ticket != stage.ticket {
                    return Err("fs_stale_ticket");
                }
                if stage.written != stage.file.len {
                    return Err("fs_incomplete");
                }
                let next = self.next(stage.generation)?;
                if self.locate(&stage.file.name).is_some() {
                    return Err("fs_exists");
                }
                let slot = self.free()?;
                // Every check precedes the sole publication. Readers never see partial bytes.
                let stage = self.stage.take().ok_or("fs_no_upload")?;
                self.files[slot] = stage.file;
                self.generation = next;
            }
            Operation::Abort(arg) => {
                let ticket = number(arg)?;
                if self.stage.as_ref().ok_or("fs_no_upload")?.ticket != ticket {
                    return Err("fs_stale_ticket");
                }
                self.stage = None;
            }
            Operation::Move(arg) | Operation::Copy(arg) => {
                let copy = matches!(op, Operation::Copy(_));
                let (generation, src, dst) = triple(arg)?;
                let next = self.next(number(generation)?)?;
                let src = Name::parse(src)?;
                let dst = Name::parse(dst)?;
                let source = self.locate(&src).ok_or("fs_not_found")?;
                if self.locate(&dst).is_some() {
                    return Err("fs_exists");
                }
                let target = if copy { self.free()? } else { source };
                let mut file = self.files[source];
                file.name = dst;
                self.files[target] = file;
                self.generation = next;
            }
        }
        Ok(self.state())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn text(store: &mut Store, op: Operation<'_>) -> String {
        format!("{}", store.execute(op).unwrap())
    }
    #[test]
    fn partial_uploads_and_failed_operations_never_publish() {
        let mut s = Store::new();
        text(&mut s, Operation::Begin("1:2:/inbox/a"));
        assert!(s.execute(Operation::Commit("1")).is_err());
        assert!(s.execute(Operation::Read("/inbox/a")).is_err());
        assert!(s.execute(Operation::Append("1:0:41zz")).is_err());
        text(&mut s, Operation::Append("1:0:4142"));
        text(&mut s, Operation::Commit("1"));
        let before = text(&mut s, Operation::Read("/inbox/a"));
        for op in [
            Operation::Move("1:/inbox/a:/work/a"),
            Operation::Copy("2:/inbox/a:/inbox/a"),
            Operation::Move("2:/inbox/a:/../escape"),
        ] {
            assert!(s.execute(op).is_err());
            assert_eq!(text(&mut s, Operation::Read("/inbox/a")), before);
        }
        text(&mut s, Operation::Copy("2:/inbox/a:/work/a"));
        assert!(text(&mut s, Operation::Read("/work/a")).ends_with("data=4142"));
        text(&mut s, Operation::Begin("3:0:/empty"));
        text(&mut s, Operation::Move("3:/work/a:/archive/a"));
        assert!(s.execute(Operation::Commit("2")).is_err());
        text(&mut s, Operation::Abort("2"));
        assert!(s.execute(Operation::Read("/empty")).is_err());
    }
    #[test]
    fn bounded_names_and_capacity() {
        for path in [
            "relative", "/", "/a/", "/a//b", "/./a", "/a/../b", "/a:b", "/a b",
        ] {
            assert!(Name::parse(path).is_err());
        }
        let mut s = Store::new();
        for i in 0..FILES {
            let arg = format!("{}:0:/f{i}", i + 1);
            text(&mut s, Operation::Begin(&arg));
            text(&mut s, Operation::Commit(&(i + 1).to_string()));
        }
        assert!(s.execute(Operation::Begin("9:0:/extra")).is_err());
        assert!(s.execute(Operation::Copy("9:/f0:/extra")).is_err());
    }
}

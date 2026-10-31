//! Bounded required revision sets and inline artifacts; no scheduling or transport.
use crate::{index_stream::Text, records::RecordId};
use core::fmt;
pub const MAX_RECORDS: usize = 4;
// Canonical lexical key order, independent of the internal numeric record IDs.
const ORDER: [u64; 6] = [6, 1, 4, 5, 2, 3];
#[derive(Clone, Copy)]
pub struct Required(u64);
impl Required {
    pub fn parse(text: &str) -> Result<Self, &'static str> {
        let mut mask = 0;
        for key in text.split(',') {
            let id = RecordId::parse(key)?;
            let bit = 1 << id.raw();
            if mask & bit != 0 {
                return Err("duplicate_required_record");
            }
            mask |= bit;
        }
        Self::from_mask(mask)
    }
    pub fn from_mask(mask: u64) -> Result<Self, &'static str> {
        if mask == 0 || mask & !126 != 0 || mask.count_ones() as usize > MAX_RECORDS {
            return Err("required_set_limit");
        }
        if mask & 12 == 12 {
            return Err("conflicting_revisions");
        }
        Ok(Self(mask))
    }
    pub fn mask(self) -> u64 {
        self.0
    }
    pub fn ids(self) -> impl Iterator<Item = RecordId> {
        ORDER
            .into_iter()
            .filter(move |id| self.0 & (1 << id) != 0)
            .map(|id| RecordId::from_raw(id).expect("validated fixed ID"))
    }
}
impl fmt::Display for Required {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (n, id) in self.ids().enumerate() {
            if n != 0 {
                f.write_str(",")?;
            }
            write!(f, "{id}")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub struct Status {
    pub generation: u32,
    pub required: Required,
    pub ready_mask: u64,
    pub completed: u64,
}
impl Status {
    pub fn ready(self) -> bool {
        self.ready_mask == self.required.mask()
    }
    pub fn ready_count(self) -> u32 {
        self.ready_mask.count_ones()
    }
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "goal_version={}; state={}; required={}; ready_records={}; total_records={}; completed={}; missing=",
            self.generation,
            if self.ready() { "complete" } else { "building" },
            self.required,
            self.ready_count(),
            self.required.mask().count_ones(),
            self.completed
        )?;
        if self.ready() {
            return f.write_str("-");
        }
        let mut first = true;
        for id in self
            .required
            .ids()
            .filter(|id| self.ready_mask & (1 << id.raw()) == 0)
        {
            if !first {
                f.write_str(",")?;
            }
            write!(f, "{id}")?;
            first = false;
        }
        Ok(())
    }
}
/// An owned immutable copy, constructed only when every required cache slot exists.
/// Its text indexes came from EL0; source bytes are the linked public read-only fixtures.
#[derive(Clone, Copy)]
pub struct Artifact {
    pub status: Status,
    pub indexes: [Text; MAX_RECORDS],
}
impl fmt::Display for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"format\":\"rai-index-set-v1\",\"goal_version\":{},\"required\":[",
            self.status.generation
        )?;
        for (n, id) in self.status.required.ids().enumerate() {
            if n != 0 {
                f.write_str(",")?;
            }
            write!(f, "\"{id}\"")?;
        }
        f.write_str("],\"records\":[")?;
        for (n, id) in self.status.required.ids().enumerate() {
            if n != 0 {
                f.write_str(",")?;
            }
            write!(
                f,
                "{{\"record\":\"{id}\",\"bytes\":{},\"source_hex\":\"",
                id.bytes()
            )?;
            for byte in id.source() {
                write!(f, "{byte:02x}")?;
            }
            // Canonical index characters contain no JSON quote or escape characters.
            write!(f, "\",\"index\":\"{}\"}}", self.indexes[n].as_str())?;
        }
        f.write_str("]}")
    }
}

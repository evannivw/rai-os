//! Pure, bounded presentation composition over the owning app's current records.
use super::{App, Capability, Kind, Spec};
use core::fmt::{self, Write};
pub const PAGES: usize = 2;
pub const BLOCKS: usize = 2;
/// Compact storage avoids usize length/padding for every small display string.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Label<const N: usize> {
    bytes: [u8; N],
    len: u8,
}
impl<const N: usize> Label<N> {
    const EMPTY: Self = Self {
        bytes: [0; N],
        len: 0,
    };
    fn new(s: &str) -> Result<Self, &'static str> {
        if s.len() > N || s.len() > 255 || !s.bytes().all(|b| (32..=126).contains(&b)) {
            return Err("page_bad_text");
        }
        let mut v = Self::EMPTY;
        v.bytes[..s.len()].copy_from_slice(s.as_bytes());
        v.len = s.len() as u8;
        Ok(v)
    }
    pub fn text(&self) -> &str {
        core::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or("")
    }
    fn hex(&self, f: &mut impl Write) -> fmt::Result {
        for b in &self.bytes[..usize::from(self.len)] {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Text,
    Metric,
    Bars,
    Table,
}
impl BlockKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Metric => "metric",
            Self::Bars => "bars",
            Self::Table => "table",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Aggregate {
    None,
    Count,
    Sum,
    Min,
    Max,
}
impl Aggregate {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Count => "count",
            Self::Sum => "sum",
            Self::Min => "min",
            Self::Max => "max",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub title: Label<18>,
    pub value: Label<64>,
    pub label_field: Label<12>,
    pub aggregate: Aggregate,
    pub pending: bool,
}
impl Block {
    const EMPTY: Self = Self {
        kind: BlockKind::Text,
        title: Label::EMPTY,
        value: Label::EMPTY,
        label_field: Label::EMPTY,
        aggregate: Aggregate::None,
        pending: false,
    };
    pub fn new(
        kind: &str,
        title: &str,
        value: &str,
        label: &str,
        aggregate: &str,
        scope: &str,
    ) -> Result<Self, &'static str> {
        let kind = match kind {
            "text" => BlockKind::Text,
            "metric" => BlockKind::Metric,
            "bars" => BlockKind::Bars,
            "table" => BlockKind::Table,
            _ => return Err("page_unknown_block"),
        };
        let aggregate = match aggregate {
            "" => Aggregate::None,
            "count" => Aggregate::Count,
            "sum" => Aggregate::Sum,
            "min" => Aggregate::Min,
            "max" => Aggregate::Max,
            _ => return Err("page_unknown_aggregate"),
        };
        let pending = match scope {
            "all" => false,
            "pending" => true,
            _ => return Err("page_bad_scope"),
        };
        if title.trim().is_empty() {
            return Err("page_bad_text");
        }
        Ok(Self {
            kind,
            title: Label::new(title)?,
            value: Label::new(value)?,
            label_field: Label::new(label)?,
            aggregate,
            pending,
        })
    }
    pub fn scope(&self) -> &'static str {
        if self.pending { "pending" } else { "all" }
    }
    fn field(spec: &Spec, key: &str) -> Result<usize, &'static str> {
        spec.fields()
            .iter()
            .position(|f| f.key.text() == key)
            .ok_or("page_unknown_field")
    }
    pub fn value_index(&self, spec: &Spec) -> Option<usize> {
        Self::field(spec, self.value.text()).ok()
    }
    pub fn label_index(&self, spec: &Spec) -> Option<usize> {
        Self::field(spec, self.label_field.text()).ok()
    }
    fn numeric(spec: &Spec, key: &str) -> Result<(), &'static str> {
        let i = Self::field(spec, key)?;
        if !matches!(spec.fields()[i].kind, Kind::Number | Kind::Calculated) {
            return Err("page_numeric_field_required");
        }
        Ok(())
    }
    fn check(&self, spec: &Spec) -> Result<(), &'static str> {
        if self.pending && spec.check_field().is_none() {
            return Err("page_filter_required");
        }
        match self.kind {
            BlockKind::Text => {
                if self.value.text().trim().is_empty()
                    || !self.label_field.text().is_empty()
                    || self.aggregate != Aggregate::None
                    || self.pending
                {
                    return Err("page_bad_block");
                }
            }
            BlockKind::Metric => {
                if !self.label_field.text().is_empty() || self.aggregate == Aggregate::None {
                    return Err("page_bad_block");
                }
                if self.aggregate == Aggregate::Count {
                    if !self.value.text().is_empty() {
                        return Err("page_bad_block");
                    }
                } else {
                    Self::numeric(spec, self.value.text())?;
                }
            }
            BlockKind::Bars | BlockKind::Table => {
                if self.aggregate != Aggregate::None {
                    return Err("page_bad_block");
                }
                Self::field(spec, self.label_field.text())?;
                if self.kind == BlockKind::Bars {
                    Self::numeric(spec, self.value.text())?;
                } else {
                    Self::field(spec, self.value.text())?;
                }
            }
        }
        Ok(())
    }
    pub fn visible(&self, app: &App, row: usize) -> bool {
        app.row(row).is_some_and(|values| {
            !self.pending
                || app
                    .spec
                    .check_field()
                    .is_some_and(|i| values[i].text() == "false")
        })
    }
    pub fn metric(&self, app: &App) -> Option<i64> {
        if self.kind != BlockKind::Metric {
            return None;
        }
        let mut result = if matches!(self.aggregate, Aggregate::Count | Aggregate::Sum) {
            Some(0)
        } else {
            None
        };
        for row in 0..app.count() {
            if !self.visible(app, row) {
                continue;
            }
            if self.aggregate == Aggregate::Count {
                result = Some(result.unwrap_or(0) + 1);
                continue;
            }
            let n = app.row(row)?[self.value_index(&app.spec)?]
                .text()
                .parse::<i32>()
                .ok();
            if let Some(n) = n {
                let n = i64::from(n);
                result = Some(match self.aggregate {
                    Aggregate::Sum => result.unwrap_or(0) + n,
                    Aggregate::Min => result.map_or(n, |r| r.min(n)),
                    Aggregate::Max => result.map_or(n, |r| r.max(n)),
                    _ => return None,
                });
            }
        }
        result
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub title: Label<24>,
    blocks: [Block; BLOCKS],
    len: u8,
}
impl Page {
    const EMPTY: Self = Self {
        title: Label::EMPTY,
        blocks: [Block::EMPTY; BLOCKS],
        len: 0,
    };
    pub fn blocks(&self) -> &[Block] {
        &self.blocks[..usize::from(self.len)]
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pages {
    pages: [Page; PAGES],
    len: u8,
}
impl Pages {
    pub const EMPTY: Self = Self {
        pages: [Page::EMPTY; PAGES],
        len: 0,
    };
    pub fn entries(&self) -> &[Page] {
        &self.pages[..usize::from(self.len)]
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn add(&mut self, title: &str) -> Result<(), &'static str> {
        if usize::from(self.len) == PAGES {
            return Err("page_limit");
        }
        if title.trim().is_empty() {
            return Err("page_bad_text");
        }
        let title = Label::new(title)?;
        if self.entries().iter().any(|p| p.title == title) {
            return Err("page_duplicate_title");
        }
        self.pages[usize::from(self.len)] = Page {
            title,
            ..Page::EMPTY
        };
        self.len += 1;
        Ok(())
    }
    pub fn add_block(&mut self, page: usize, block: Block) -> Result<(), &'static str> {
        let p = self
            .pages
            .get_mut(page)
            .filter(|_| page < usize::from(self.len))
            .ok_or("page_not_found")?;
        if usize::from(p.len) == BLOCKS {
            return Err("page_block_limit");
        }
        p.blocks[usize::from(p.len)] = block;
        p.len += 1;
        Ok(())
    }
    pub fn check(&self, spec: &Spec) -> Result<(), &'static str> {
        if !self.is_empty() && spec.capability() != Capability::Records {
            return Err("page_capability_denied");
        }
        for p in self.entries() {
            if p.blocks().is_empty() {
                return Err("page_empty");
            }
            for b in p.blocks() {
                b.check(spec)?;
            }
        }
        Ok(())
    }
}
impl fmt::Display for Pages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, p) in self.entries().iter().enumerate() {
            if i > 0 {
                f.write_str("|")?;
            }
            p.title.hex(f)?;
            for b in p.blocks() {
                write!(f, "~{}^", b.kind.name())?;
                b.title.hex(f)?;
                f.write_str("^")?;
                b.value.hex(f)?;
                write!(
                    f,
                    "^{}^{}^{}",
                    b.label_field.text(),
                    b.aggregate.name(),
                    b.scope()
                )?;
            }
        }
        Ok(())
    }
}
/// Diagnostic values are independently testable; the renderer uses the same pure
/// metrics. Empty min/max is explicit, and sums of <=8 i32 values fit in i64.
pub struct PageValues<'a>(pub &'a App, pub usize);
impl fmt::Display for PageValues<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = &self.0.spec.pages.entries()[self.1];
        f.write_str("metrics=")?;
        for (i, b) in p.blocks().iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            if b.kind == BlockKind::Metric {
                if let Some(n) = b.metric(self.0) {
                    write!(f, "{n}")?;
                } else {
                    f.write_str("empty")?;
                }
            } else {
                f.write_str("-")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microapp::{Field, Runtime, Text};
    fn spec() -> Spec {
        let mut s = Spec::new("report", "Report", "records.local").unwrap();
        s.add(Field::new("name", "Name", "text", true, "Item").unwrap())
            .unwrap();
        s.add(Field::new("value", "Value", "number", false, "").unwrap())
            .unwrap();
        s.add(Field::new("done", "Done", "check", true, "false").unwrap())
            .unwrap();
        s.filter("done", "Pending").unwrap();
        s
    }
    fn block(aggregate: &str, scope: &str) -> Block {
        Block::new(
            "metric",
            "Result",
            if aggregate == "count" { "" } else { "value" },
            "",
            aggregate,
            scope,
        )
        .unwrap()
    }
    #[test]
    fn aggregates_cover_blank_signed_empty_and_filtered_data_without_mutation() {
        let s = spec();
        let mut r = Runtime::new();
        let i = r.install(&s, r.revision(), None).unwrap();
        assert_eq!(block("min", "all").metric(r.app(i).unwrap()), None);
        assert_eq!(block("sum", "all").metric(r.app(i).unwrap()), Some(0));
        for (n, done) in [
            ("-9", "false"),
            ("20", "true"),
            ("", "false"),
            ("4", "false"),
        ] {
            let mut v = r.app(i).unwrap().defaults();
            v[1] = Text::new(n).unwrap();
            v[2] = Text::new(done).unwrap();
            r.save(i, None, &v).unwrap();
        }
        let revision = r.revision();
        let a = r.app(i).unwrap();
        for (op, scope, expected) in [
            ("count", "all", 4),
            ("count", "pending", 3),
            ("sum", "all", 15),
            ("sum", "pending", -5),
            ("min", "all", -9),
            ("max", "pending", 4),
        ] {
            assert_eq!(block(op, scope).metric(a), Some(expected));
        }
        assert_eq!(r.revision(), revision);
        assert_eq!(a.row(0).unwrap()[1].text(), "-9");
    }
    #[test]
    fn full_capacity_aggregate_is_wider_than_cell_arithmetic() {
        let s = spec();
        let mut r = Runtime::new();
        let i = r.install(&s, r.revision(), None).unwrap();
        for _ in 0..8 {
            let mut v = r.app(i).unwrap().defaults();
            v[1] = Text::new("2147483647").unwrap();
            r.save(i, None, &v).unwrap();
        }
        assert_eq!(
            block("sum", "all").metric(r.app(i).unwrap()),
            Some(17179869176)
        );
    }
    #[test]
    fn invalid_bindings_scopes_and_device_capabilities_are_rejected() {
        let s = spec();
        for (kind, value, label, aggregate, error) in [
            ("metric", "name", "", "sum", "page_numeric_field_required"),
            ("bars", "missing", "name", "", "page_unknown_field"),
            ("table", "value", "missing", "", "page_unknown_field"),
            ("text", "hello", "name", "", "page_bad_block"),
            ("metric", "value", "", "count", "page_bad_block"),
        ] {
            assert_eq!(
                Block::new(kind, "Title", value, label, aggregate, "all")
                    .unwrap()
                    .check(&s),
                Err(error)
            );
        }
        let mut no_filter = s;
        no_filter.filter("", "").unwrap();
        assert_eq!(
            block("sum", "pending").check(&no_filter),
            Err("page_filter_required")
        );
        assert!(Block::new("script", "Title", "", "", "", "all").is_err());
        assert!(Block::new("text", "Title", &"x".repeat(65), "", "", "all").is_err());
        let mut display = Spec::new("light", "Light", "display.brightness").unwrap();
        display
            .add(Field::new("level", "Level", "number", true, "100").unwrap())
            .unwrap();
        display.pages.add("Report").unwrap();
        assert_eq!(display.check(), Err("page_capability_denied"));
    }
    #[test]
    fn page_limits_and_wire_escaping_preserve_arbitrary_printable_text() {
        let mut s = spec();
        s.pages.add("Overview").unwrap();
        assert_eq!(s.check(), Err("page_empty"));
        let b = Block::new(
            "text",
            "A^|~",
            "Words | with ~ delimiters ^ and :",
            "",
            "",
            "all",
        )
        .unwrap();
        s.pages.add_block(0, b).unwrap();
        s.pages.add_block(0, block("sum", "all")).unwrap();
        assert_eq!(s.pages.add_block(0, b), Err("page_block_limit"));
        assert_eq!(s.pages.add("Overview"), Err("page_duplicate_title"));
        s.pages.add("Details").unwrap();
        s.pages.add_block(1, b).unwrap();
        assert_eq!(s.pages.add("Third"), Err("page_limit"));
        assert!(s.check().is_ok());
        let encoded = s.pages.to_string();
        assert_eq!(encoded.split('|').count(), 2);
        assert!(!encoded.contains("Words"));
        assert!(core::mem::size_of::<Pages>() < 512);
    }
    #[test]
    fn presentation_updates_keep_records_and_recompute_from_current_values() {
        let mut s = spec();
        s.pages.add("Total").unwrap();
        s.pages.add_block(0, block("sum", "all")).unwrap();
        let mut r = Runtime::new();
        let i = r.install(&s, r.revision(), None).unwrap();
        let mut row = r.app(i).unwrap().defaults();
        row[1] = Text::new("14").unwrap();
        r.save(i, None, &row).unwrap();
        let mut next = s;
        next.pages = Pages::EMPTY;
        next.pages.add("Maximum").unwrap();
        next.pages.add_block(0, block("max", "all")).unwrap();
        r.install(&next, r.revision(), Some(i)).unwrap();
        assert_eq!(r.app(i).unwrap().row(0).unwrap()[1].text(), "14");
        assert_eq!(
            next.pages.entries()[0].blocks()[0].metric(r.app(i).unwrap()),
            Some(14)
        );
        row[1] = Text::new("-3").unwrap();
        r.save(i, Some(0), &row).unwrap();
        assert_eq!(
            next.pages.entries()[0].blocks()[0].metric(r.app(i).unwrap()),
            Some(-3)
        );
    }
}

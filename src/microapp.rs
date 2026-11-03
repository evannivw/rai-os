//! Bounded declarative records runtime. No I/O, bytecode, pointers or ambient capabilities.
use core::fmt::{self, Write};
pub const APPS: usize = 3;
pub const FIELDS: usize = 6;
pub const ROWS: usize = 8;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Text<const N: usize> {
    bytes: [u8; N],
    len: usize,
}
impl<const N: usize> Text<N> {
    pub const EMPTY: Self = Self {
        bytes: [0; N],
        len: 0,
    };
    pub fn new(s: &str) -> Result<Self, &'static str> {
        if s.len() > N || !s.bytes().all(|b| (32..=126).contains(&b)) {
            return Err("app_bad_text");
        }
        let mut t = Self::EMPTY;
        t.bytes[..s.len()].copy_from_slice(s.as_bytes());
        t.len = s.len();
        Ok(t)
    }
    pub fn text(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
    pub fn push(&mut self, b: u8) {
        if self.len < N && (32..=126).contains(&b) {
            self.bytes[self.len] = b;
            self.len += 1;
        }
    }
    pub fn pop(&mut self) {
        self.len = self.len.saturating_sub(1);
    }
    pub fn hex(&self, f: &mut impl Write) -> fmt::Result {
        for b in &self.bytes[..self.len] {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}
pub fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 12
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Number,
    Check,
}
impl Kind {
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        match s {
            "text" => Ok(Self::Text),
            "number" => Ok(Self::Number),
            "check" => Ok(Self::Check),
            _ => Err("app_unknown_kind"),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Check => "check",
        }
    }
    fn valid(self, value: &str) -> bool {
        match self {
            Self::Text => true,
            Self::Number => value.is_empty() || value.parse::<i32>().is_ok(),
            Self::Check => matches!(value, "true" | "false"),
        }
    }
}
#[derive(Clone, Copy)]
pub struct Field {
    pub key: Text<12>,
    pub label: Text<22>,
    pub kind: Kind,
    pub required: bool,
    pub default: Text<32>,
}
impl Field {
    const EMPTY: Self = Self {
        key: Text::EMPTY,
        label: Text::EMPTY,
        kind: Kind::Text,
        required: false,
        default: Text::EMPTY,
    };
    pub fn new(
        key: &str,
        label: &str,
        kind: &str,
        required: bool,
        default: &str,
    ) -> Result<Self, &'static str> {
        if !identifier(key) || label.trim().is_empty() {
            return Err("app_bad_field");
        }
        let result = Self {
            key: Text::new(key)?,
            label: Text::new(label)?,
            kind: Kind::parse(kind)?,
            required,
            default: Text::new(default)?,
        };
        if !result.kind.valid(default) {
            return Err("app_bad_default");
        }
        Ok(result)
    }
    pub fn valid(&self, value: &str) -> bool {
        (!self.required || !value.trim().is_empty()) && self.kind.valid(value)
    }
}
#[derive(Clone, Copy)]
pub struct Spec {
    pub id: Text<12>,
    pub title: Text<28>,
    fields: [Field; FIELDS],
    len: usize,
    pub filter_key: Text<12>,
    pub filter_label: Text<18>,
}
impl Spec {
    pub const EMPTY: Self = Self {
        id: Text::EMPTY,
        title: Text::EMPTY,
        fields: [Field::EMPTY; FIELDS],
        len: 0,
        filter_key: Text::EMPTY,
        filter_label: Text::EMPTY,
    };
    pub fn new(id: &str, title: &str, capability: &str) -> Result<Self, &'static str> {
        if capability != "records.local" {
            return Err("app_capability_denied");
        }
        if !identifier(id) || title.trim().is_empty() {
            return Err("app_bad_identity");
        }
        Ok(Self {
            id: Text::new(id)?,
            title: Text::new(title)?,
            ..Self::EMPTY
        })
    }
    pub fn add(&mut self, f: Field) -> Result<(), &'static str> {
        if self.len == FIELDS {
            return Err("app_field_limit");
        }
        if self.fields().iter().any(|a| a.key == f.key) {
            return Err("app_duplicate_field");
        }
        self.fields[self.len] = f;
        self.len += 1;
        Ok(())
    }
    pub fn filter(&mut self, key: &str, label: &str) -> Result<(), &'static str> {
        if key.is_empty() != label.is_empty() {
            return Err("app_bad_filter");
        }
        self.filter_key = Text::new(key)?;
        self.filter_label = Text::new(label)?;
        Ok(())
    }
    pub fn fields(&self) -> &[Field] {
        &self.fields[..self.len]
    }
    pub fn check(&self) -> Result<(), &'static str> {
        if self.len == 0 || !identifier(self.id.text()) {
            return Err("app_empty_spec");
        }
        if !self.filter_key.text().is_empty() && self.check_field().is_none() {
            return Err("app_bad_filter");
        }
        Ok(())
    }
    pub fn check_field(&self) -> Option<usize> {
        self.fields()
            .iter()
            .position(|f| f.kind == Kind::Check && f.key == self.filter_key)
    }
    pub fn compatible(&self, old: &Spec) -> Result<(), &'static str> {
        if self.id != old.id || self.len < old.len {
            return Err("app_destructive_update");
        }
        for (a, b) in self.fields().iter().zip(old.fields()) {
            if a.key != b.key || a.kind != b.kind {
                return Err("app_destructive_update");
            }
        }
        Ok(())
    }
}
impl fmt::Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id={}; title=", self.id.text())?;
        self.title.hex(f)?;
        write!(f, "; filter={}; filter_label=", self.filter_key.text())?;
        self.filter_label.hex(f)?;
        f.write_str("; fields=")?;
        for (i, a) in self.fields().iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(
                f,
                "{}~{}~{}~",
                a.key.text(),
                a.kind.name(),
                u8::from(a.required)
            )?;
            a.label.hex(f)?;
            f.write_str("~")?;
            a.default.hex(f)?;
        }
        Ok(())
    }
}
pub type Values = [Text<32>; FIELDS];
#[derive(Clone, Copy)]
pub struct App {
    pub spec: Spec,
    pub version: u32,
    rows: [Values; ROWS],
    count: usize,
}
impl App {
    const EMPTY: Self = Self {
        spec: Spec::EMPTY,
        version: 0,
        rows: [[Text::EMPTY; FIELDS]; ROWS],
        count: 0,
    };
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn row(&self, index: usize) -> Option<&Values> {
        self.rows[..self.count].get(index)
    }
    pub fn defaults(&self) -> Values {
        let mut v = [Text::EMPTY; FIELDS];
        for (i, f) in self.spec.fields().iter().enumerate() {
            v[i] = f.default;
        }
        v
    }
}
pub struct Runtime {
    apps: [App; APPS],
    revision: u32,
}
impl Runtime {
    pub const fn new() -> Self {
        Self {
            apps: [App::EMPTY; APPS],
            revision: 1,
        }
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn app(&self, index: usize) -> Option<&App> {
        self.apps.get(index).filter(|a| a.version > 0)
    }
    pub fn entries(&self) -> impl Iterator<Item = (usize, &App)> {
        self.apps.iter().enumerate().filter(|(_, a)| a.version > 0)
    }
    pub fn find(&self, id: &str) -> Option<usize> {
        self.entries()
            .find(|(_, a)| a.spec.id.text() == id)
            .map(|(i, _)| i)
    }
    pub fn validate(
        &self,
        spec: &Spec,
        base: u32,
        target: Option<usize>,
    ) -> Result<usize, &'static str> {
        if self.revision != base {
            return Err("app_stale_revision");
        }
        self.revision
            .checked_add(1)
            .ok_or("app_revision_exhausted")?;
        spec.check()?;
        if let Some(index) = target {
            let old = self.app(index).ok_or("app_not_found")?;
            spec.compatible(&old.spec)?;
            old.version.checked_add(1).ok_or("app_version_exhausted")?;
            for row in &old.rows[..old.count] {
                for (i, field) in spec.fields().iter().enumerate() {
                    let v = if i < old.spec.len {
                        row[i]
                    } else {
                        field.default
                    };
                    if !field.valid(v.text()) {
                        return Err("app_migration_requires_value");
                    }
                }
            }
            Ok(index)
        } else {
            if self.find(spec.id.text()).is_some() {
                return Err("app_identity_taken");
            }
            self.apps
                .iter()
                .position(|a| a.version == 0)
                .ok_or("app_slot_limit")
        }
    }
    pub fn install(
        &mut self,
        spec: &Spec,
        base: u32,
        target: Option<usize>,
    ) -> Result<usize, &'static str> {
        let slot = self.validate(spec, base, target)?;
        let app = &mut self.apps[slot];
        // Validation precedes all writes. Existing columns keep exact bytes; new columns
        // use checked defaults. No descriptor primitive addresses another app's rows.
        for row in &mut app.rows[..app.count] {
            for (i, field) in spec.fields().iter().enumerate().skip(app.spec.len) {
                row[i] = field.default;
            }
        }
        app.spec = *spec;
        app.version += 1;
        self.revision += 1;
        Ok(slot)
    }
    pub fn save(
        &mut self,
        index: usize,
        row: Option<usize>,
        values: &Values,
    ) -> Result<(), &'static str> {
        let app = self.app(index).ok_or("app_not_found")?;
        let slot = if let Some(r) = row {
            if r >= app.count {
                return Err("app_row_not_found");
            }
            r
        } else {
            if app.count == ROWS {
                return Err("app_row_limit");
            }
            app.count
        };
        for (i, field) in app.spec.fields().iter().enumerate() {
            if !field.valid(values[i].text()) {
                return Err("app_invalid_value");
            }
        }
        self.revision
            .checked_add(1)
            .ok_or("app_revision_exhausted")?;
        let app = &mut self.apps[index];
        app.rows[slot] = *values;
        if row.is_none() {
            app.count += 1;
        }
        self.revision += 1;
        Ok(())
    }
    pub fn toggle(&mut self, index: usize, row: usize) -> Result<(), &'static str> {
        let app = self.app(index).ok_or("app_not_found")?;
        let field = app.spec.check_field().ok_or("app_no_check_filter")?;
        let mut values = *app.row(row).ok_or("app_row_not_found")?;
        values[field] = Text::new(if values[field].text() == "true" {
            "false"
        } else {
            "true"
        })?;
        self.save(index, Some(row), &values)
    }
}
/// Bounded aggregation over validated signed-32-bit cells. Eight cells fit i64.
impl App {
    pub fn sum(&self, field: usize, pending: bool) -> Option<i64> {
        if self.spec.fields().get(field)?.kind != Kind::Number {
            return None;
        }
        Some(
            (0..self.count)
                .filter_map(|row| {
                    let values = self.row(row)?;
                    if pending
                        && self
                            .spec
                            .check_field()
                            .is_some_and(|i| values[i].text() == "true")
                    {
                        return None;
                    }
                    values[field].text().parse::<i32>().ok().map(i64::from)
                })
                .sum(),
        )
    }
}
pub struct Totals<'a>(pub &'a App, pub bool);
impl fmt::Display for Totals<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id={}; scope={}; totals=",
            self.0.spec.id.text(),
            if self.1 { "pending" } else { "all" }
        )?;
        let mut first = true;
        for (i, field) in self.0.spec.fields().iter().enumerate() {
            if let Some(value) = self.0.sum(i, self.1) {
                if !first {
                    f.write_str(",")?;
                }
                first = false;
                write!(f, "{}:{value}", field.key.text())?;
            }
        }
        if first {
            f.write_str("-")?;
        }
        Ok(())
    }
}
pub struct Catalog<'a>(pub &'a Runtime);
impl fmt::Display for Catalog<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "revision={}; apps=", self.0.revision())?;
        let mut first = true;
        for (_, app) in self.0.entries() {
            if !first {
                f.write_str(",")?;
            }
            first = false;
            write!(f, "{}:{}:{}:", app.spec.id.text(), app.version, app.count)?;
            app.spec.title.hex(f)?;
        }
        if first {
            f.write_str("-")?;
        }
        Ok(())
    }
}
pub struct Export<'a>(pub &'a App);
impl fmt::Display for Export<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id={}; version={}; count={}; rows=",
            self.0.spec.id.text(),
            self.0.version,
            self.0.count
        )?;
        for (i, row) in self.0.rows[..self.0.count].iter().enumerate() {
            if i > 0 {
                f.write_str("|")?;
            }
            for (j, value) in row[..self.0.spec.len].iter().enumerate() {
                if j > 0 {
                    f.write_str(",")?;
                }
                value.hex(f)?;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn spec(id: &str) -> Spec {
        let mut s = Spec::new(id, "Inspections", "records.local").unwrap();
        s.add(Field::new("equipment", "Equipment", "text", true, "").unwrap())
            .unwrap();
        s.add(Field::new("done", "Done", "check", false, "false").unwrap())
            .unwrap();
        s.filter("done", "Pending").unwrap();
        s
    }
    #[test]
    fn totals_use_wide_integer_arithmetic_and_the_selected_filter() {
        let mut s = spec("sum");
        s.add(Field::new("amount", "Amount", "number", false, "").unwrap())
            .unwrap();
        let mut r = Runtime::new();
        let i = r.install(&s, r.revision(), None).unwrap();
        for n in 0..8 {
            let mut values = r.app(i).unwrap().defaults();
            values[0] = Text::new("Row").unwrap();
            values[2] = Text::new("2147483647").unwrap();
            values[1] = Text::new(if n < 3 { "true" } else { "false" }).unwrap();
            r.save(i, None, &values).unwrap();
        }
        let a = r.app(i).unwrap();
        assert_eq!(a.sum(2, false), Some(17_179_869_176));
        assert_eq!(a.sum(2, true), Some(10_737_418_235));
        assert_eq!(a.sum(0, false), None);
        let mut row = *a.row(0).unwrap();
        row[2] = Text::EMPTY;
        r.save(i, Some(0), &row).unwrap();
        assert_eq!(r.app(i).unwrap().sum(2, false), Some(15_032_385_529));
    }
    #[test]
    fn scope_migration_and_failed_updates() {
        let mut r = Runtime::new();
        let s = spec("inspect");
        let a = r.install(&s, 1, None).unwrap();
        let b = r.install(&spec("stock"), 2, None).unwrap();
        let mut v = r.app(a).unwrap().defaults();
        v[0] = Text::new("Pump A").unwrap();
        r.save(a, None, &v).unwrap();
        assert_eq!(r.app(b).unwrap().count(), 0);
        let before = format!("{}", Export(r.app(a).unwrap()));
        let mut update = s;
        update
            .add(Field::new("priority", "Priority", "number", true, "1").unwrap())
            .unwrap();
        r.install(&update, 4, Some(a)).unwrap();
        assert_eq!(r.app(a).unwrap().row(0).unwrap()[0].text(), "Pump A");
        assert_eq!(r.app(a).unwrap().row(0).unwrap()[2].text(), "1");
        assert!(r.install(&s, 5, Some(a)).is_err());
        assert!(r.install(&spec("stock"), 5, Some(a)).is_err());
        assert!(r.install(&update, 4, Some(a)).is_err());
        let now = format!("{}", Export(r.app(a).unwrap()));
        let mut bad = update;
        bad.add(Field::new("needed", "Needed", "text", true, "").unwrap())
            .unwrap();
        assert!(r.install(&bad, 5, Some(a)).is_err());
        assert_eq!(format!("{}", Export(r.app(a).unwrap())), now);
        assert!(before.contains("50756d702041"));
        r.toggle(a, 0).unwrap();
        assert_eq!(r.app(a).unwrap().row(0).unwrap()[1].text(), "true");
        assert_eq!(r.app(b).unwrap().count(), 0);
    }
    #[test]
    fn bounded_and_denied_capabilities() {
        assert!(Spec::new("a", "A", "network").is_err());
        assert!(Field::new("a", "A", "shell", false, "").is_err());
        let mut s = spec("a");
        assert!(
            s.add(Field::new("done", "Other", "text", false, "").unwrap())
                .is_err()
        );
        let mut r = Runtime::new();
        let a = r.install(&s, 1, None).unwrap();
        let mut v = r.app(a).unwrap().defaults();
        assert!(r.save(a, None, &v).is_err());
        v[0] = Text::new("A").unwrap();
        for _ in 0..ROWS {
            r.save(a, None, &v).unwrap();
        }
        assert!(r.save(a, None, &v).is_err());
        assert!(r.save(3, None, &v).is_err());
        assert!(r.save(a, Some(8), &v).is_err());
    }
}

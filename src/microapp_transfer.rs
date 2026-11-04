//! Same-app checked transfers. All reads and validation precede both writes.
use super::{Capability, Kind, Runtime, Spec, Text, Values, identifier, integer};
use core::fmt;
#[derive(Clone, Copy)]
pub struct Transfer {
    pub id: Text<12>,
    pub label: Text<22>,
    pub balance_field: Text<12>,
    pub amount_field: Text<12>,
    pub target_field: Text<12>,
    pub key_field: Text<12>,
}
impl Transfer {
    pub fn new(
        id: &str,
        label: &str,
        balance: &str,
        amount: &str,
        target: &str,
        key: &str,
    ) -> Result<Self, &'static str> {
        if ![id, balance, amount, target, key]
            .iter()
            .all(|s| identifier(s))
            || label.trim().is_empty()
        {
            return Err("move_bad_rule");
        }
        Ok(Self {
            id: Text::new(id)?,
            label: Text::new(label)?,
            balance_field: Text::new(balance)?,
            amount_field: Text::new(amount)?,
            target_field: Text::new(target)?,
            key_field: Text::new(key)?,
        })
    }
    fn indices(&self, spec: &Spec) -> Result<[usize; 4], &'static str> {
        let keys = [
            self.balance_field,
            self.amount_field,
            self.target_field,
            self.key_field,
        ];
        let mut indices = [0; 4];
        for (i, key) in keys.iter().enumerate() {
            indices[i] = spec
                .fields()
                .iter()
                .position(|f| f.key == *key)
                .ok_or("move_bad_binding")?;
            let f = &spec.fields()[indices[i]];
            if f.kind != if i < 2 { Kind::Number } else { Kind::Text }
                || ((i == 0 || i == 3) && !f.required)
                || keys[..i].contains(key)
            {
                return Err("move_bad_binding");
            }
        }
        Ok(indices)
    }
    pub fn check(&self, spec: &Spec) -> Result<(), &'static str> {
        if spec.capability() != Capability::Records {
            return Err("move_capability_denied");
        }
        if !identifier(self.id.text()) || self.label.text().trim().is_empty() {
            return Err("move_bad_rule");
        }
        self.indices(spec)?;
        Ok(())
    }
}
impl fmt::Display for Transfer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}~", self.id.text())?;
        self.label.hex(f)?;
        write!(
            f,
            "~{}~{}~{}~{}",
            self.balance_field.text(),
            self.amount_field.text(),
            self.target_field.text(),
            self.key_field.text()
        )
    }
}
pub struct TransferPreview {
    pub target: usize,
    pub amount: i32,
    pub source_before: i32,
    pub target_before: i32,
    pub source_after: i32,
    pub target_after: i32,
    pub source_key: Text<32>,
    pub target_key: Text<32>,
    source_values: Values,
    target_values: Values,
}
impl Runtime {
    pub fn preview_transfer(
        &self,
        index: usize,
        row: usize,
        id: &str,
        base: u32,
    ) -> Result<TransferPreview, &'static str> {
        if base != self.revision {
            return Err("app_stale_revision");
        }
        self.revision
            .checked_add(1)
            .ok_or("app_revision_exhausted")?;
        let app = self.app(index).ok_or("app_not_found")?;
        let rule = app
            .spec
            .transfer
            .as_ref()
            .filter(|r| r.id.text() == id)
            .ok_or("move_unknown_rule")?;
        rule.check(&app.spec)?;
        let [balance, amount, target, key] = rule.indices(&app.spec)?;
        let source = app.row(row).ok_or("app_row_not_found")?;
        if source[target].text().trim().is_empty() {
            return Err("move_target_missing");
        }
        let mut matches = (0..app.count).filter(|i| app.rows[*i][key] == source[target]);
        let destination = matches.next().ok_or("move_target_missing")?;
        if matches.next().is_some() {
            return Err("move_target_ambiguous");
        }
        if destination == row {
            return Err("move_same_row");
        }
        if app.spec.workflow.terminal(app.states[row])
            || app.spec.workflow.terminal(app.states[destination])
        {
            return Err("workflow_closed");
        }
        let dest = &app.rows[destination];
        let parse = |v: Text<32>| v.text().parse::<i32>().map_err(|_| "move_invalid_number");
        let amount = parse(source[amount])?;
        let source_before = parse(source[balance])?;
        let target_before = parse(dest[balance])?;
        if amount <= 0 || source_before < 0 || target_before < 0 {
            return Err("move_invalid_number");
        }
        if source_before < amount {
            return Err("move_insufficient");
        }
        let source_after = source_before - amount;
        let target_after = target_before.checked_add(amount).ok_or("move_overflow")?;
        let mut source_values = *source;
        let mut target_values = *dest;
        source_values[balance] = integer(source_after);
        target_values[balance] = integer(target_after);
        // Derived columns and all fields must succeed in BOTH candidate rows.
        let source_values = app.spec.materialize(&source_values)?;
        let target_values = app.spec.materialize(&target_values)?;
        Ok(TransferPreview {
            target: destination,
            amount,
            source_before,
            target_before,
            source_after,
            target_after,
            source_key: source[key],
            target_key: dest[key],
            source_values,
            target_values,
        })
    }
    pub fn run_transfer(
        &mut self,
        index: usize,
        row: usize,
        id: &str,
        base: u32,
    ) -> Result<(), &'static str> {
        let p = self.preview_transfer(index, row, id, base)?;
        // No fallible operation remains. The single-threaded app runtime has no
        // interrupt writers; both writes become visible before the next command.
        // This is RAM atomicity, not a durable filesystem transaction.
        self.apps[index].rows[row] = p.source_values;
        self.apps[index].rows[p.target] = p.target_values;
        self.revision += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microapp::{Action, Field, Workflow};
    fn spec() -> Spec {
        let mut s = Spec::new("ledger", "Ledger", "records.local").unwrap();
        for (key, kind, default) in [
            ("name", "text", ""),
            ("stock", "number", "0"),
            ("amount", "number", "0"),
            ("to", "text", ""),
        ] {
            s.add(Field::new(key, key, kind, key != "to", default).unwrap())
                .unwrap();
        }
        s.transfer =
            Some(Transfer::new("send", "Transfer", "stock", "amount", "to", "name").unwrap());
        s
    }
    fn runtime(s: &Spec) -> Runtime {
        let mut r = Runtime::new();
        r.install(s, 1, None).unwrap();
        for (name, stock, amount, target) in [("A", "7", "2", "B"), ("B", "3", "0", "")] {
            let mut v = [Text::EMPTY; 6];
            for (i, t) in [name, stock, amount, target].iter().enumerate() {
                v[i] = Text::new(t).unwrap();
            }
            r.save(0, None, &v).unwrap();
        }
        r
    }
    fn set(r: &mut Runtime, row: usize, field: usize, value: &str) {
        let mut v = *r.app(0).unwrap().row(row).unwrap();
        v[field] = Text::new(value).unwrap();
        r.save(0, Some(row), &v).unwrap();
    }
    fn rejected(r: &mut Runtime, error: &str) {
        let before = r.apps[0].rows;
        let revision = r.revision();
        assert_eq!(r.run_transfer(0, 0, "send", revision), Err(error));
        assert!(before == r.apps[0].rows);
        assert_eq!(r.revision(), revision);
    }
    #[test]
    fn transfer_resolves_a_destination_renamed_to_a_shorter_key() {
        let mut r = runtime(&spec());
        let mut destination = *r.app(0).unwrap().row(1).unwrap();
        destination[0] = Text::new("Dispatch").unwrap();
        r.save(0, Some(1), &destination).unwrap();
        for _ in 0..8 {
            destination[0].pop();
        }
        destination[0].push(b'B');
        r.save(0, Some(1), &destination).unwrap();
        let revision = r.revision();
        r.run_transfer(0, 0, "send", revision).unwrap();
        assert_eq!(r.app(0).unwrap().row(0).unwrap()[1].text(), "5");
        assert_eq!(r.app(0).unwrap().row(1).unwrap()[1].text(), "5");
        assert_eq!(r.revision(), revision + 1);
    }

    #[test]
    fn transfer_preview_and_commit_conserve_total_and_recompute_both_rows() {
        let mut s = spec();
        s.add(
            Field::new("double", "Double", "calculated", true, "")
                .unwrap()
                .with_formula("$stock*2")
                .unwrap(),
        )
        .unwrap();
        let mut r = runtime(&s);
        let revision = r.revision();
        let p = r.preview_transfer(0, 0, "send", revision).unwrap();
        assert_eq!((p.source_after, p.target_after, p.amount), (5, 5, 2));
        assert_eq!(r.revision(), revision);
        assert_eq!(r.app(0).unwrap().row(0).unwrap()[1].text(), "7");
        r.run_transfer(0, 0, "send", revision).unwrap();
        assert_eq!(r.revision(), revision + 1);
        for row in 0..2 {
            assert_eq!(r.app(0).unwrap().row(row).unwrap()[1].text(), "5");
            assert_eq!(r.app(0).unwrap().row(row).unwrap()[4].text(), "10");
        }
        assert_eq!(
            r.run_transfer(0, 0, "send", revision),
            Err("app_stale_revision")
        );
    }
    #[test]
    fn transfer_rejects_missing_ambiguous_self_and_invalid_balances_without_any_write() {
        for (field, value, error) in [
            (3, "Z", "move_target_missing"),
            (3, "", "move_target_missing"),
            (3, "A", "move_same_row"),
            (2, "0", "move_invalid_number"),
            (2, "-1", "move_invalid_number"),
            (2, "8", "move_insufficient"),
            (1, "-1", "move_invalid_number"),
        ] {
            let mut r = runtime(&spec());
            set(&mut r, 0, field, value);
            rejected(&mut r, error);
        }
        let mut r = runtime(&spec());
        set(&mut r, 1, 1, "2147483647");
        rejected(&mut r, "move_overflow");
        set(&mut r, 1, 1, "-1");
        rejected(&mut r, "move_invalid_number");
        let row = *r.app(0).unwrap().row(1).unwrap();
        r.save(0, None, &row).unwrap();
        rejected(&mut r, "move_target_ambiguous");
    }
    #[test]
    fn transfer_late_derived_error_and_terminal_rows_abort_both_writes() {
        let mut s = spec();
        s.add(
            Field::new("inverse", "Inverse", "calculated", true, "")
                .unwrap()
                .with_formula("10/($stock-5)")
                .unwrap(),
        )
        .unwrap();
        let mut r = runtime(&s);
        set(&mut r, 0, 1, "8"); // source 8->6 valid, destination 3->5 invalid
        let before = r.apps[0].rows;
        let revision = r.revision();
        assert!(r.run_transfer(0, 0, "send", revision).is_err());
        assert!(before == r.apps[0].rows);
        assert_eq!(revision, r.revision());
        let mut s = spec();
        s.workflow = Workflow::new("open").unwrap();
        s.workflow
            .add(Action::new("close", "Close", "open", "closed", "", "none", 0).unwrap())
            .unwrap();
        for row in 0..2 {
            let mut r = runtime(&s);
            r.apply_action(0, row, "close", r.revision()).unwrap();
            rejected(&mut r, "workflow_closed");
        }
    }
    #[test]
    fn transfer_bindings_and_capability_are_checked_at_admission() {
        let mut s = spec();
        s.transfer.as_mut().unwrap().target_field = Text::new("name").unwrap();
        assert_eq!(s.check(), Err("move_bad_binding"));
        let mut s = spec();
        s.transfer.as_mut().unwrap().balance_field = Text::new("to").unwrap();
        assert_eq!(s.check(), Err("move_bad_binding"));
        let mut s = spec();
        s.transfer.as_mut().unwrap().id = Text::EMPTY;
        assert_eq!(s.check(), Err("move_bad_rule"));
        let mut s = Spec::new("screen", "Screen", "display.brightness").unwrap();
        s.add(Field::new("level", "Level", "number", true, "80").unwrap())
            .unwrap();
        s.transfer = spec().transfer;
        assert_eq!(s.check(), Err("move_capability_denied"));
    }
}

//! Small, typed button programs over one saved record. No I/O or ambient authority.
//! All writes stay in a local candidate until every executed step has succeeded.
use super::{Capability, Kind, Spec, Text, Values, formula, identifier, integer};

pub const BUTTONS: usize = 2;
pub const STEPS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BatchError {
    /// One-based failing row, or zero for batch admission failure.
    pub row: usize,
    pub error: BehaviorError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BehaviorError {
    /// One-based failed instruction, or zero for request admission failure.
    pub step: u8,
    pub code: &'static str,
}
impl BehaviorError {
    pub fn admission(code: &'static str) -> Self {
        Self { step: 0, code }
    }
    fn instruction(step: usize, code: &'static str) -> Self {
        Self {
            step: (step + 1) as u8,
            code,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub op: Text<12>,
    pub field: Text<12>,
    pub value: Text<64>,
    pub compare: Text<2>,
    pub skip: usize,
}
impl Step {
    const EMPTY: Self = Self {
        op: Text::EMPTY,
        field: Text::EMPTY,
        value: Text::EMPTY,
        compare: Text::EMPTY,
        skip: 0,
    };
    pub fn new(
        op: &str,
        field: &str,
        value: &str,
        compare: &str,
        skip: usize,
    ) -> Result<Self, &'static str> {
        if !identifier(field) || !matches!(op, "set" | "calculate" | "toggle" | "require" | "if") {
            return Err("behavior_bad_step");
        }
        if matches!(op, "require" | "if") {
            if !matches!(compare, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
                return Err("behavior_bad_comparison");
            }
        } else if !compare.is_empty() {
            return Err("behavior_bad_comparison");
        }
        if (op == "if" && !(1..=STEPS).contains(&skip)) || (op != "if" && skip != 0) {
            return Err("behavior_bad_skip");
        }
        if (op == "toggle" && !value.is_empty()) || (op != "calculate" && value.len() > 32) {
            return Err("behavior_bad_value");
        }
        Ok(Self {
            op: Text::new(op)?,
            field: Text::new(field)?,
            value: Text::new(value)?,
            compare: Text::new(compare)?,
            skip,
        })
    }
    fn index(&self, spec: &Spec) -> Result<usize, &'static str> {
        spec.fields()
            .iter()
            .position(|f| f.key == self.field)
            .ok_or("behavior_unknown_field")
    }
    fn program(&self, spec: &Spec) -> Result<formula::Program, &'static str> {
        formula::compile(self.value.text(), |key| {
            spec.fields().iter().position(|f| {
                f.key.text() == key && matches!(f.kind, Kind::Number | Kind::Calculated)
            })
        })
    }
    fn mutates(&self) -> bool {
        matches!(self.op.text(), "set" | "calculate" | "toggle")
    }
    fn check(&self, spec: &Spec) -> Result<(), &'static str> {
        // Validate again: the public fields are inspectable by presentation code,
        // so schema admission must not rely on callers having used new().
        Self::new(
            self.op.text(),
            self.field.text(),
            self.value.text(),
            self.compare.text(),
            self.skip,
        )?;
        let field = &spec.fields()[self.index(spec)?];
        match self.op.text() {
            "set" => {
                if field.kind == Kind::Calculated {
                    return Err("behavior_read_only_field");
                }
                if !field.valid(self.value.text()) {
                    return Err("behavior_bad_value");
                }
            }
            "calculate" => {
                if field.kind != Kind::Number {
                    return Err("behavior_number_required");
                }
                self.program(spec)?;
            }
            "toggle" => {
                if field.kind != Kind::Check {
                    return Err("behavior_check_required");
                }
            }
            "if" | "require" => {
                if matches!(field.kind, Kind::Number | Kind::Calculated) {
                    self.value
                        .text()
                        .parse::<i32>()
                        .map_err(|_| "behavior_bad_value")?;
                } else {
                    if !matches!(self.compare.text(), "eq" | "ne") {
                        return Err("behavior_bad_comparison");
                    }
                    if field.kind == Kind::Check && !matches!(self.value.text(), "true" | "false") {
                        return Err("behavior_bad_value");
                    }
                }
            }
            _ => return Err("behavior_bad_step"),
        }
        Ok(())
    }
    fn condition(&self, spec: &Spec, values: &Values) -> Result<bool, &'static str> {
        let index = self.index(spec)?;
        let comparison = if matches!(spec.fields()[index].kind, Kind::Number | Kind::Calculated) {
            let left = values[index]
                .text()
                .parse::<i32>()
                .map_err(|_| "behavior_missing_number")?;
            let right = self
                .value
                .text()
                .parse::<i32>()
                .map_err(|_| "behavior_bad_value")?;
            left.cmp(&right)
        } else {
            values[index].text().cmp(self.value.text())
        };
        use core::cmp::Ordering::{Equal, Greater, Less};
        Ok(match self.compare.text() {
            "eq" => comparison == Equal,
            "ne" => comparison != Equal,
            "lt" => comparison == Less,
            "le" => comparison != Greater,
            "gt" => comparison == Greater,
            "ge" => comparison != Less,
            _ => return Err("behavior_bad_comparison"),
        })
    }
    fn write(&self, spec: &Spec, values: &mut Values) -> Result<(), &'static str> {
        let index = self.index(spec)?;
        values[index] = match self.op.text() {
            "set" => Text::new(self.value.text())?,
            "toggle" => Text::new(if values[index].text() == "true" {
                "false"
            } else {
                "true"
            })?,
            "calculate" => {
                let mut numbers = [None; super::FIELDS];
                for (i, field) in spec.fields().iter().enumerate() {
                    if matches!(field.kind, Kind::Number | Kind::Calculated) {
                        numbers[i] = values[i].text().parse::<i32>().ok();
                    }
                }
                integer(self.program(spec)?.eval(&numbers)?)
            }
            _ => return Err("behavior_bad_step"),
        };
        // Later steps observe new derived values, not the old calculated cells.
        *values = spec.materialize(values)?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Button {
    pub all_rows: bool,
    pub id: Text<12>,
    pub label: Text<22>,
    pub event: Text<8>,
    steps: [Step; STEPS],
    len: usize,
}
impl Button {
    const EMPTY: Self = Self {
        all_rows: false,
        id: Text::EMPTY,
        label: Text::EMPTY,
        event: Text::EMPTY,
        steps: [Step::EMPTY; STEPS],
        len: 0,
    };
    pub fn new(id: &str, label: &str, event: &str) -> Result<Self, &'static str> {
        if !identifier(id) || label.trim().is_empty() || event != "press" {
            return Err("behavior_bad_button");
        }
        Ok(Self {
            id: Text::new(id)?,
            label: Text::new(label)?,
            event: Text::new(event)?,
            ..Self::EMPTY
        })
    }
    #[cfg(not(target_os = "none"))]
    pub fn with_scope(mut self, scope: &str) -> Result<Self, &'static str> {
        self.all_rows = match scope {
            "row" => false,
            "all" => true,
            _ => return Err("behavior_bad_scope"),
        };
        Ok(self)
    }
    pub fn steps(&self) -> &[Step] {
        &self.steps[..self.len]
    }
    pub fn add(&mut self, step: Step) -> Result<(), &'static str> {
        if self.len == STEPS {
            return Err("behavior_step_limit");
        }
        self.steps[self.len] = step;
        self.len += 1;
        Ok(())
    }
    fn check(&self, spec: &Spec) -> Result<(), &'static str> {
        Self::new(self.id.text(), self.label.text(), self.event.text())?;
        if self.len == 0 || !self.steps().iter().any(Step::mutates) {
            return Err("behavior_no_change");
        }
        for (i, step) in self.steps().iter().enumerate() {
            step.check(spec)?;
            if step.op.text() == "if" && step.skip > self.len - i - 1 {
                return Err("behavior_bad_skip");
            }
        }
        Ok(())
    }
    pub fn preview(&self, spec: &Spec, values: &Values) -> Result<Values, BehaviorError> {
        self.check(spec).map_err(BehaviorError::admission)?;
        let mut candidate = *values;
        let mut index = 0;
        // Every instruction advances the program counter; if may only skip
        // forward. At most STEPS instructions execute, with no recursion/loops.
        while index < self.len {
            let step = &self.steps[index];
            let result = match step.op.text() {
                "require" | "if" => match step.condition(spec, &candidate) {
                    Ok(false) if step.op.text() == "require" => Err("behavior_condition_failed"),
                    Ok(false) => {
                        index += step.skip;
                        Ok(())
                    }
                    Ok(true) => Ok(()),
                    Err(error) => Err(error),
                },
                _ => step.write(spec, &mut candidate),
            };
            if let Err(error) = result {
                return Err(BehaviorError::instruction(index, error));
            }
            index += 1;
        }
        Ok(candidate)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Behaviors {
    buttons: [Button; BUTTONS],
    len: usize,
}
impl Behaviors {
    pub const EMPTY: Self = Self {
        buttons: [Button::EMPTY; BUTTONS],
        len: 0,
    };
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn buttons(&self) -> &[Button] {
        &self.buttons[..self.len]
    }
    pub fn add(&mut self, button: Button) -> Result<(), &'static str> {
        if self.len == BUTTONS {
            return Err("behavior_button_limit");
        }
        if self.buttons().iter().any(|b| b.id == button.id) {
            return Err("behavior_duplicate_button");
        }
        self.buttons[self.len] = button;
        self.len += 1;
        Ok(())
    }
    pub fn set_scope(&mut self, id: &str, scope: &str) -> Result<(), &'static str> {
        let button = self.buttons[..self.len]
            .iter_mut()
            .find(|b| b.id.text() == id)
            .ok_or("behavior_unknown_button")?;
        // Parse before changing the stored button.
        let all_rows = match scope {
            "row" => false,
            "all" => true,
            _ => return Err("behavior_bad_scope"),
        };
        button.all_rows = all_rows;
        Ok(())
    }
    pub fn add_step(&mut self, id: &str, step: Step) -> Result<(), &'static str> {
        self.buttons[..self.len]
            .iter_mut()
            .find(|b| b.id.text() == id)
            .ok_or("behavior_unknown_button")?
            .add(step)
    }
    pub fn check(&self, spec: &Spec) -> Result<(), &'static str> {
        if !self.is_empty() && spec.capability() != Capability::Records {
            return Err("behavior_capability_denied");
        }
        for button in self.buttons() {
            button.check(spec)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microapp::{Action, Field, Runtime, Workflow};

    fn spec() -> Spec {
        let mut s = Spec::new("ledger", "Resource ledger", "records.local").unwrap();
        for (key, kind, default) in [
            ("name", "text", "Box"),
            ("stock", "number", "5"),
            ("moved", "number", "0"),
            ("flag", "check", "false"),
            ("status", "text", "normal"),
            ("total", "calculated", ""),
        ] {
            s.add(Field::new(key, key, kind, true, default).unwrap())
                .unwrap();
        }
        s.formula("total", "$stock+$moved").unwrap();
        s
    }
    fn step(op: &str, field: &str, value: &str, compare: &str, skip: usize) -> Step {
        Step::new(op, field, value, compare, skip).unwrap()
    }
    fn button(id: &str, steps: &[Step]) -> Button {
        let mut b = Button::new(id, id, "press").unwrap();
        for &s in steps {
            b.add(s).unwrap();
        }
        b
    }
    fn setup(s: &Spec) -> (Runtime, usize) {
        let mut r = Runtime::new();
        let slot = r.install(s, r.revision(), None).unwrap();
        let defaults = r.app(slot).unwrap().defaults();
        r.save(slot, None, &defaults).unwrap();
        (r, slot)
    }
    fn cell(r: &Runtime, slot: usize, i: usize) -> &str {
        r.app(slot).unwrap().row(0).unwrap()[i].text()
    }
    #[test]
    fn composed_steps_preview_atomically_and_observe_updated_derived_values() {
        let mut s = spec();
        s.behaviors
            .add(button(
                "move",
                &[
                    step("require", "stock", "1", "ge", 0),
                    step("calculate", "stock", "$stock-1", "", 0),
                    step("calculate", "moved", "$moved+1", "", 0),
                    step("toggle", "flag", "", "", 0),
                    step("if", "stock", "2", "lt", 1),
                    step("set", "status", "low", "", 0),
                ],
            ))
            .unwrap();
        s.behaviors
            .add(button(
                "derive",
                &[
                    // A behavior expression may refer to a later derived field.
                    step("calculate", "stock", "$total+1", "", 0),
                    // The previous step refreshed total before this condition.
                    step("require", "total", "6", "eq", 0),
                ],
            ))
            .unwrap();
        let (mut r, slot) = setup(&s);
        let before = *r.app(slot).unwrap().row(0).unwrap();
        let rev = r.revision();
        let preview = r.preview_behavior(slot, 0, "move", rev).unwrap();
        assert_eq!(preview[1].text(), "4");
        assert_eq!(preview[2].text(), "1");
        assert_eq!(preview[3].text(), "true");
        assert_eq!(preview[4].text(), "normal");
        assert_eq!(preview[5].text(), "5");
        assert!(*r.app(slot).unwrap().row(0).unwrap() == before);
        assert_eq!(r.revision(), rev);
        r.run_behavior(slot, 0, "derive", rev).unwrap();
        assert_eq!(cell(&r, slot, 1), "6");
        assert_eq!(r.revision(), rev + 1);
        for _ in 0..5 {
            r.run_behavior(slot, 0, "move", r.revision()).unwrap();
        }
        assert_eq!(cell(&r, slot, 1), "1");
        assert_eq!(cell(&r, slot, 2), "5");
        assert_eq!(cell(&r, slot, 4), "low");
        assert_eq!(cell(&r, slot, 5), "6");
    }
    #[test]
    fn late_condition_and_arithmetic_failures_publish_nothing() {
        for (failing, code) in [
            (
                step("require", "stock", "0", "eq", 0),
                "behavior_condition_failed",
            ),
            (step("calculate", "stock", "1/0", "", 0), "formula_div_zero"),
            (
                step("calculate", "stock", "2147483647+1", "", 0),
                "formula_overflow",
            ),
        ] {
            let mut s = spec();
            s.behaviors
                .add(button(
                    "fail",
                    &[step("set", "status", "changed", "", 0), failing],
                ))
                .unwrap();
            let (mut r, slot) = setup(&s);
            let before = *r.app(slot).unwrap().row(0).unwrap();
            let rev = r.revision();
            let error = r.run_behavior(slot, 0, "fail", rev).unwrap_err();
            assert_eq!(error, BehaviorError { step: 2, code });
            assert!(*r.app(slot).unwrap().row(0).unwrap() == before);
            assert_eq!(r.revision(), rev);
        }
    }
    #[test]
    fn all_branches_types_references_and_forward_bounds_are_validated() {
        for (bad, error) in [
            (step("set", "total", "9", "", 0), "behavior_read_only_field"),
            (step("set", "missing", "9", "", 0), "behavior_unknown_field"),
            (step("set", "flag", "yes", "", 0), "behavior_bad_value"),
            (
                step("calculate", "name", "1", "", 0),
                "behavior_number_required",
            ),
            (
                step("toggle", "stock", "", "", 0),
                "behavior_check_required",
            ),
            (
                step("require", "name", "Box", "lt", 0),
                "behavior_bad_comparison",
            ),
            (step("if", "stock", "2", "eq", 1), "behavior_bad_skip"),
        ] {
            let mut s = spec();
            // The first condition is false on the sample record, but its skipped
            // step is still rejected during installation.
            s.behaviors
                .add(button(
                    "bad",
                    &[
                        step("set", "status", "ready", "", 0),
                        step("if", "stock", "0", "lt", 1),
                        bad,
                    ],
                ))
                .unwrap();
            assert_eq!(s.check(), Err(error));
        }
        assert!(Step::new("shell", "stock", "anything", "", 0).is_err());
        assert!(Step::new("set", "stock", "1", "eq", 0).is_err());
        assert!(Step::new("if", "stock", "1", "eq", 0).is_err());
        assert!(Step::new("require", "stock", "1", "eq", 1).is_err());
        assert!(Step::new("toggle", "flag", "true", "", 0).is_err());
        assert!(Button::new("auto", "Automatic", "timer").is_err());
        let mut s = spec();
        s.behaviors
            .add(button("observe", &[step("require", "stock", "1", "ge", 0)]))
            .unwrap();
        assert_eq!(s.check(), Err("behavior_no_change"));
    }
    #[test]
    fn conditions_compare_typed_values_and_missing_numbers_are_errors() {
        let mut s = spec();
        s.behaviors
            .add(button(
                "typed",
                &[
                    step("require", "name", "Box", "eq", 0),
                    step("require", "flag", "false", "eq", 0),
                    step("if", "status", "normal", "ne", 1),
                    step("set", "name", "Skipped", "", 0),
                    step("toggle", "flag", "", "", 0),
                ],
            ))
            .unwrap();
        let (mut r, slot) = setup(&s);
        r.run_behavior(slot, 0, "typed", r.revision()).unwrap();
        assert_eq!(cell(&r, slot, 0), "Box");
        assert_eq!(cell(&r, slot, 3), "true");
        let before = r.revision();
        assert_eq!(
            r.run_behavior(slot, 0, "typed", before).unwrap_err(),
            BehaviorError {
                step: 2,
                code: "behavior_condition_failed"
            }
        );
        assert_eq!(r.revision(), before);
        let mut optional = Spec::new("empty", "Optional", "records.local").unwrap();
        optional
            .add(Field::new("count", "Count", "number", false, "").unwrap())
            .unwrap();
        optional
            .behaviors
            .add(button(
                "check",
                &[
                    step("require", "count", "0", "eq", 0),
                    step("set", "count", "1", "", 0),
                ],
            ))
            .unwrap();
        let (mut r, slot) = setup(&optional);
        let before = r.revision();
        assert_eq!(
            r.run_behavior(slot, 0, "check", before).unwrap_err(),
            BehaviorError {
                step: 1,
                code: "behavior_missing_number"
            }
        );
        assert_eq!(r.revision(), before);
    }
    #[test]
    fn terminal_records_stale_revisions_and_wrong_targets_cannot_mutate() {
        let mut s = spec();
        s.behaviors
            .add(button("edit", &[step("set", "status", "new", "", 0)]))
            .unwrap();
        s.workflow = Workflow::new("open").unwrap();
        s.workflow
            .add(Action::new("close", "Close", "open", "closed", "", "none", 0).unwrap())
            .unwrap();
        let (mut r, slot) = setup(&s);
        let other = r.install(&spec(), r.revision(), None);
        assert_eq!(other, Err("app_identity_taken"));
        for (index, row, id, revision, code) in [
            (slot, 0, "edit", r.revision() - 1, "app_stale_revision"),
            (slot, 1, "edit", r.revision(), "app_row_not_found"),
            (99, 0, "edit", r.revision(), "app_not_found"),
            (slot, 0, "unknown", r.revision(), "behavior_unknown_button"),
        ] {
            let before = r.revision();
            assert_eq!(
                r.run_behavior(index, row, id, revision),
                Err(BehaviorError::admission(code))
            );
            assert_eq!(r.revision(), before);
        }
        r.apply_action(slot, 0, "close", r.revision()).unwrap();
        let before = r.revision();
        assert_eq!(
            r.run_behavior(slot, 0, "edit", before),
            Err(BehaviorError::admission("workflow_closed"))
        );
        assert_eq!(cell(&r, slot, 4), "normal");
        assert_eq!(r.revision(), before);
    }
    #[test]
    fn reviewed_behavior_updates_preserve_records_and_other_apps() {
        let old = spec();
        let (mut r, slot) = setup(&old);
        let original = *r.app(slot).unwrap().row(0).unwrap();
        let mut other = spec();
        other.id = Text::new("other").unwrap();
        let other_slot = r.install(&other, r.revision(), None).unwrap();
        let values = r.app(other_slot).unwrap().defaults();
        r.save(other_slot, None, &values).unwrap();
        let mut updated = old;
        updated
            .behaviors
            .add(button("rename", &[step("set", "name", "New", "", 0)]))
            .unwrap();
        r.install(&updated, r.revision(), Some(slot)).unwrap();
        assert!(*r.app(slot).unwrap().row(0).unwrap() == original);
        r.run_behavior(slot, 0, "rename", r.revision()).unwrap();
        assert_eq!(cell(&r, slot, 0), "New");
        assert!(*r.app(other_slot).unwrap().row(0).unwrap() == original);
        let mut invalid = updated;
        invalid.behaviors = Behaviors::EMPTY;
        invalid
            .behaviors
            .add(button("invalid", &[step("set", "total", "999", "", 0)]))
            .unwrap();
        let before = r.revision();
        assert_eq!(
            r.install(&invalid, before, Some(slot)),
            Err("behavior_read_only_field")
        );
        assert_eq!(r.revision(), before);
        assert!(r.app(slot).unwrap().spec.behaviors == updated.behaviors);
        let mut display = Spec::new("screen", "Screen", "display.brightness").unwrap();
        display
            .add(Field::new("level", "Level", "number", true, "50").unwrap())
            .unwrap();
        display
            .behaviors
            .add(button("set", &[step("set", "level", "80", "", 0)]))
            .unwrap();
        assert_eq!(display.check(), Err("behavior_capability_denied"));
    }
    #[test]
    fn public_step_fields_cannot_bypass_schema_admission() {
        let valid = step("set", "name", "A", "", 0);
        for mutation in 0..7 {
            let mut bad = valid;
            match mutation {
                0 => bad.op = Text::new("shell").unwrap(),
                1 => bad.field = Text::new("other.app").unwrap(),
                2 => bad.skip = usize::MAX,
                3 => bad.compare = Text::new("ge").unwrap(),
                4 => bad.field = Text::new("total").unwrap(),
                5 => {
                    bad.op = Text::new("toggle").unwrap();
                    bad.field = Text::new("flag").unwrap();
                }
                _ => {
                    bad.op = Text::new("if").unwrap();
                    bad.field = Text::new("stock").unwrap();
                    bad.compare = Text::new("eq").unwrap();
                    bad.skip = usize::MAX;
                }
            }
            let mut s = spec();
            s.behaviors.add(button("bad", &[valid, bad])).unwrap();
            assert!(s.check().is_err());
        }
        let mut s = spec();
        let mut b = button("bad", &[valid]);
        b.event = Text::new("timer").unwrap();
        s.behaviors.add(b).unwrap();
        assert_eq!(s.check(), Err("behavior_bad_button"));
    }
    #[test]
    fn program_counts_and_wire_format_are_bounded() {
        let one = step("set", "name", "A,^/~;", "", 0);
        let mut b = button("fill", &[one; STEPS]);
        assert_eq!(b.add(one), Err("behavior_step_limit"));
        let mut s = spec();
        s.behaviors.add(b).unwrap();
        assert_eq!(s.behaviors.add(b), Err("behavior_duplicate_button"));
        s.behaviors.add(button("other", &[one])).unwrap();
        assert_eq!(
            s.behaviors.add(button("third", &[one])),
            Err("behavior_button_limit")
        );
        s.check().unwrap();
        let wire = s.to_string();
        assert!(wire.contains("behaviors=fill~66696c6c~press~set^name^412c5e2f7e3b^^0/"));
        assert!(
            Spec::new("empty", "Empty", "records.local")
                .unwrap()
                .to_string()
                .ends_with("; behaviors=")
        );
        assert!(core::mem::size_of::<Behaviors>() <= 2048);
    }
    #[test]
    fn all_rows_preview_branches_and_one_revision_commit_at_capacity() {
        let mut s = spec();
        s.behaviors
            .add(
                button(
                    "batch",
                    &[
                        step("if", "stock", "2", "ge", 3),
                        step("calculate", "stock", "$stock-2", "", 0),
                        step("calculate", "moved", "$moved+2", "", 0),
                        step("set", "status", "moved", "", 0),
                    ],
                )
                .with_scope("all")
                .unwrap(),
            )
            .unwrap();
        let mut r = Runtime::new();
        let slot = r.install(&s, r.revision(), None).unwrap();
        assert_eq!(
            r.preview_behavior_all(slot, "batch", r.revision())
                .unwrap_err()
                .error
                .code,
            "behavior_empty_batch"
        );
        for i in 0..crate::microapp::ROWS {
            let mut values = r.app(slot).unwrap().defaults();
            values[1] = integer(i as i32);
            r.save(slot, None, &values).unwrap();
        }
        let rev = r.revision();
        assert_eq!(r.preview_behavior_all(slot, "batch", rev), Ok(6));
        assert_eq!(r.revision(), rev);
        for i in 0..8 {
            assert_eq!(
                r.app(slot).unwrap().row(i).unwrap()[1].text(),
                i.to_string()
            );
        }
        assert_eq!(
            r.run_behavior(slot, 0, "batch", rev).unwrap_err().code,
            "behavior_scope_required"
        );
        r.run_behavior_all(slot, "batch", rev).unwrap();
        assert_eq!(r.revision(), rev + 1);
        for i in 0..8 {
            let v = r.app(slot).unwrap().row(i).unwrap();
            assert_eq!(v[1].text(), (if i < 2 { i } else { i - 2 }).to_string());
            assert_eq!(v[2].text(), if i < 2 { "0" } else { "2" });
            assert_eq!(v[4].text(), if i < 2 { "normal" } else { "moved" });
            assert_eq!(v[5].text(), i.to_string());
        }
        assert_eq!(
            r.run_behavior_all(slot, "batch", rev)
                .unwrap_err()
                .error
                .code,
            "app_stale_revision"
        );
    }
    #[test]
    fn late_row_errors_rollback_the_entire_batch_and_terminal_locks_apply() {
        for (operation, code) in [
            (
                step("require", "stock", "1", "ge", 0),
                "behavior_condition_failed",
            ),
            (
                step("calculate", "moved", "10/$stock", "", 0),
                "formula_div_zero",
            ),
            (
                step("calculate", "moved", "2147483647+$stock", "", 0),
                "formula_overflow",
            ),
        ] {
            let mut s = spec();
            s.behaviors
                .add(
                    button(
                        "batch",
                        &[step("set", "status", "staged", "", 0), operation],
                    )
                    .with_scope("all")
                    .unwrap(),
                )
                .unwrap();
            let mut r = Runtime::new();
            let slot = r.install(&s, r.revision(), None).unwrap();
            for i in 0..8 {
                let mut v = r.app(slot).unwrap().defaults();
                v[1] = Text::new(if code == "formula_overflow" {
                    if i == 7 { "1" } else { "0" }
                } else if i == 7 {
                    "0"
                } else {
                    "5"
                })
                .unwrap();
                r.save(slot, None, &v).unwrap();
            }
            let before = *r.app(slot).unwrap();
            let rev = r.revision();
            let error = r.run_behavior_all(slot, "batch", rev).unwrap_err();
            assert_eq!(error.row, 8);
            assert_eq!(error.error.step, 2);
            assert_eq!(error.error.code, code);
            assert_eq!(r.revision(), rev);
            for i in 0..8 {
                assert!(r.app(slot).unwrap().row(i) == before.row(i));
            }
        }
        let mut s = spec();
        s.workflow = Workflow::new("open").unwrap();
        s.workflow
            .add(Action::new("close", "Close", "open", "closed", "", "none", 0).unwrap())
            .unwrap();
        s.behaviors
            .add(
                button("batch", &[step("set", "status", "staged", "", 0)])
                    .with_scope("all")
                    .unwrap(),
            )
            .unwrap();
        s.behaviors
            .add(button("row", &[step("set", "status", "local", "", 0)]))
            .unwrap();
        let (mut r, slot) = setup(&s);
        let v = r.app(slot).unwrap().defaults();
        r.save(slot, None, &v).unwrap();
        r.apply_action(slot, 1, "close", r.revision()).unwrap();
        let rev = r.revision();
        let error = r.run_behavior_all(slot, "batch", rev).unwrap_err();
        assert_eq!(error.row, 2);
        assert_eq!(error.error.code, "workflow_closed");
        assert_eq!(cell(&r, slot, 4), "normal");
        assert_eq!(r.revision(), rev);
        assert_eq!(
            r.run_behavior_all(slot, "row", rev).unwrap_err().error.code,
            "behavior_scope_required"
        );
        assert!(
            Button::new("bad", "Bad", "press")
                .unwrap()
                .with_scope("filtered")
                .is_err()
        );
    }
}

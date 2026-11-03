//! Declarative, user-triggered state transitions. No scheduling or external effects.
use super::{Kind, Spec, Text, Values, identifier};
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Action {
    pub id: Text<12>,
    pub label: Text<22>,
    pub from: Text<12>,
    pub to: Text<12>,
    pub guard_field: Text<12>,
    pub guard_op: Text<4>,
    pub guard_value: i32,
}
impl Action {
    const EMPTY: Self = Self {
        id: Text::EMPTY,
        label: Text::EMPTY,
        from: Text::EMPTY,
        to: Text::EMPTY,
        guard_field: Text::EMPTY,
        guard_op: Text::EMPTY,
        guard_value: 0,
    };
    pub fn new(
        id: &str,
        label: &str,
        from: &str,
        to: &str,
        guard_field: &str,
        guard_op: &str,
        guard_value: i32,
    ) -> Result<Self, &'static str> {
        if !identifier(id)
            || !identifier(from)
            || !identifier(to)
            || from == to
            || label.trim().is_empty()
        {
            return Err("workflow_bad_action");
        }
        if (guard_op == "none" && (!guard_field.is_empty() || guard_value != 0))
            || (!matches!(guard_op, "none" | "eq" | "le" | "ge"))
            || (guard_op != "none" && !identifier(guard_field))
        {
            return Err("workflow_bad_guard");
        }
        Ok(Self {
            id: Text::new(id)?,
            label: Text::new(label)?,
            from: Text::new(from)?,
            to: Text::new(to)?,
            guard_field: Text::new(guard_field)?,
            guard_op: Text::new(guard_op)?,
            guard_value,
        })
    }
    pub fn allowed(
        &self,
        spec: &Spec,
        values: &Values,
        state: Text<12>,
    ) -> Result<(), &'static str> {
        if state != self.from {
            return Err("workflow_wrong_state");
        }
        if self.guard_op.text() == "none" {
            return Ok(());
        }
        let i = spec
            .fields()
            .iter()
            .position(|f| f.key == self.guard_field)
            .ok_or("workflow_bad_guard")?;
        let value = values[i]
            .text()
            .parse::<i32>()
            .map_err(|_| "workflow_guard_failed")?;
        let yes = match self.guard_op.text() {
            "eq" => value == self.guard_value,
            "le" => value <= self.guard_value,
            "ge" => value >= self.guard_value,
            _ => false,
        };
        if yes {
            Ok(())
        } else {
            Err("workflow_guard_failed")
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Workflow {
    pub initial: Text<12>,
    actions: [Action; 4],
    len: usize,
}
impl Workflow {
    pub const EMPTY: Self = Self {
        initial: Text::EMPTY,
        actions: [Action::EMPTY; 4],
        len: 0,
    };
    pub fn new(initial: &str) -> Result<Self, &'static str> {
        if !identifier(initial) {
            return Err("workflow_bad_state");
        }
        Ok(Self {
            initial: Text::new(initial)?,
            ..Self::EMPTY
        })
    }
    pub fn enabled(&self) -> bool {
        !self.initial.text().is_empty()
    }
    pub fn actions(&self) -> &[Action] {
        &self.actions[..self.len]
    }
    pub fn add(&mut self, action: Action) -> Result<(), &'static str> {
        if !self.enabled() || self.len == 4 {
            return Err("workflow_action_limit");
        }
        if self.actions().iter().any(|a| a.id == action.id) {
            return Err("workflow_duplicate_action");
        }
        self.actions[self.len] = action;
        self.len += 1;
        Ok(())
    }
    pub fn known(&self, state: Text<12>) -> bool {
        state == self.initial || self.actions().iter().any(|a| a.to == state)
    }
    pub fn terminal(&self, state: Text<12>) -> bool {
        self.enabled() && !self.actions().iter().any(|a| a.from == state)
    }
    pub fn check(&self, spec: &Spec) -> Result<(), &'static str> {
        if !self.enabled() {
            return Ok(());
        }
        if self.len == 0 || spec.capability() != super::Capability::Records {
            return Err("workflow_invalid");
        }
        let mut reachable = [Text::EMPTY; 5];
        reachable[0] = self.initial;
        let mut len = 1;
        for _ in 0..4 {
            for a in self.actions() {
                if a.guard_op.text() != "none"
                    && !spec.fields().iter().any(|f| {
                        f.key == a.guard_field && matches!(f.kind, Kind::Number | Kind::Calculated)
                    })
                {
                    return Err("workflow_bad_guard");
                }
                if reachable[..len].contains(&a.from) && !reachable[..len].contains(&a.to) {
                    reachable[len] = a.to;
                    len += 1;
                }
            }
        }
        if self
            .actions()
            .iter()
            .any(|a| !reachable[..len].contains(&a.from))
        {
            return Err("workflow_unreachable");
        }
        Ok(())
    }
}

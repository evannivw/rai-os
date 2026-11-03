//! Guest UI controller for a bounded interpreter. No model or transport dependency.
use crate::microapp::{
    Action, BehaviorError, Brightness, Button, Capability, Field, ImportBatch, Kind, Runtime, Spec,
    Step, Text, Values, Workflow,
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Home,
    Search,
    Compose,
    Thinking,
    Review,
    Form,
    Records,
    Table,
    Notice,
    Transfer,
    ImportReview,
    Workflow,
    WorkflowReview,
    Behaviors,
    BehaviorReview,
    BehaviorPreview,
}
pub struct Controller {
    pub active: bool,
    pub live: bool,
    pub mode: Mode,
    pub runtime: Runtime,
    pub selected: usize,
    pub focus: usize,
    pub row: usize,
    pub columns: usize,
    pub filtered: bool,
    pub draft: Values,
    pub editing: Option<usize>,
    pub prompt: Text<128>,
    pub search: Text<32>,
    pub storage: Text<16>,
    pub brightness: Brightness,
    pub display_available: bool,
    pub message: Text<160>,
    pub pending: Spec,
    pub import: ImportBatch,
    pub behavior_error: Option<BehaviorError>,
    pub behavior_failed_row: usize,
    pub behavior_changed: usize,
    behavior_base: u32,
    behavior_id: Text<12>,
    importing: bool,
    restoring: bool,
    restore_started: bool,
    request: u32,
    base: u32,
    target: Option<usize>,
}
pub enum Reply {
    Transfer {
        request: u32,
        revision: u32,
        app: Text<12>,
        import: bool,
    },
    State(&'static str),
    Submit {
        request: u32,
        revision: u32,
        target: Text<12>,
        prompt: Text<128>,
    },
}
impl core::fmt::Display for Reply {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer {
                request,
                revision,
                app,
                import,
            } => write!(
                f,
                "transfer={}; request={request}; revision={revision}; app={}",
                if *import { "import" } else { "export" },
                app.text()
            ),
            Self::State(s) => write!(f, "state={s}"),
            Self::Submit {
                request,
                revision,
                target,
                prompt,
            } => {
                write!(
                    f,
                    "submit={request}; revision={revision}; target={}; prompt=",
                    if target.text().is_empty() {
                        "-"
                    } else {
                        target.text()
                    }
                )?;
                prompt.hex(f)
            }
        }
    }
}
fn parts<const N: usize>(s: &str) -> Result<[&str; N], &'static str> {
    let mut result = [""; N];
    let mut i = s.split(':');
    for v in &mut result {
        *v = i.next().ok_or("bad_args")?;
    }
    if i.next().is_some() {
        return Err("bad_args");
    }
    Ok(result)
}
fn decode<const N: usize>(s: &str) -> Result<Text<N>, &'static str> {
    if !s.len().is_multiple_of(2) || s.len() > N * 2 {
        return Err("app_bad_hex");
    }
    let mut b = [0u8; N];
    for (i, p) in s.as_bytes().chunks_exact(2).enumerate() {
        let digit = |c: u8| match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            _ => Err("app_bad_hex"),
        };
        b[i] = digit(p[0])? * 16 + digit(p[1])?;
    }
    Text::new(core::str::from_utf8(&b[..s.len() / 2]).map_err(|_| "app_bad_text")?)
}
impl Controller {
    pub const fn new() -> Self {
        Self {
            active: false,
            live: false,
            mode: Mode::Home,
            runtime: Runtime::new(),
            selected: 0,
            focus: 0,
            row: 0,
            columns: 0,
            filtered: false,
            draft: [Text::EMPTY; 6],
            editing: None,
            prompt: Text::EMPTY,
            search: Text::EMPTY,
            storage: Text::EMPTY,
            brightness: Brightness::FULL,
            display_available: false,
            message: Text::EMPTY,
            pending: Spec::EMPTY,
            import: ImportBatch::EMPTY,
            behavior_error: None,
            behavior_failed_row: 0,
            behavior_changed: 0,
            behavior_base: 0,
            behavior_id: Text::EMPTY,
            importing: false,
            restoring: false,
            restore_started: false,
            request: 0,
            base: 0,
            target: None,
        }
    }
    #[cfg(feature = "guest-display")]
    pub fn target(&self) -> Option<usize> {
        self.target
    }
    #[cfg(feature = "guest-display")]
    pub fn behavior_label(&self) -> &str {
        self.runtime
            .app(self.selected)
            .and_then(|a| {
                a.spec
                    .behaviors
                    .buttons()
                    .iter()
                    .find(|b| b.id == self.behavior_id)
            })
            .map_or("Button", |b| b.label.text())
    }
    pub fn behavior_is_all(&self) -> bool {
        self.runtime.app(self.selected).is_some_and(|a| {
            a.spec
                .behaviors
                .buttons()
                .iter()
                .any(|b| b.id == self.behavior_id && b.all_rows)
        })
    }
    fn state(&self) -> &'static str {
        match self.mode {
            Mode::Home => "home",
            Mode::Search => "search",
            Mode::Compose => "compose",
            Mode::Thinking => "thinking",
            Mode::Review => "review",
            Mode::Form => "form",
            Mode::Records => "records",
            Mode::Table => "table",
            Mode::Notice => "notice",
            Mode::Transfer => "transfer",
            Mode::ImportReview => "import_review",
            Mode::Workflow => "workflow",
            Mode::WorkflowReview => "workflow_review",
            Mode::Behaviors => "behaviors",
            Mode::BehaviorReview => "behavior_review",
            Mode::BehaviorPreview => "behavior_preview",
        }
    }
    pub fn app_matches(&self, slot: usize) -> bool {
        self.runtime.app(slot).is_some_and(|a| {
            let query = self.search.text().as_bytes();
            query.is_empty()
                || [a.spec.title.text(), a.spec.id.text()].iter().any(|value| {
                    value
                        .as_bytes()
                        .windows(query.len())
                        .any(|part| part.eq_ignore_ascii_case(query))
                })
        })
    }
    fn select_match(&mut self) {
        if !self.app_matches(self.selected) {
            self.selected = (0..crate::microapp::APPS)
                .find(|i| self.app_matches(*i))
                .unwrap_or(0);
        }
    }
    fn tap(&mut self, x: usize, y: usize) -> Result<Reply, &'static str> {
        if x >= 480 || y >= 800 {
            return Err("app_bad_pointer");
        }
        if y >= 754 {
            if x < 160 {
                return self.key(27);
            }
            if x < 320 {
                return self.key(1002);
            }
            self.key(27)?;
            return self.key(47);
        }
        if (696..745).contains(&y) {
            return self.key(13);
        }
        if self.mode == Mode::Table && x >= 350 && (635..670).contains(&y) {
            return self.key(1008);
        }
        if self.mode == Mode::ImportReview && (195..235).contains(&y) {
            return self.key(1006);
        }
        if self.mode == Mode::Form
            && self
                .runtime
                .app(self.selected)
                .is_some_and(|a| a.spec.capability() == Capability::Brightness)
            && (330..382).contains(&y)
        {
            return self.key(1010);
        }
        if matches!(
            self.mode,
            Mode::Form
                | Mode::Records
                | Mode::Table
                | Mode::Workflow
                | Mode::Behaviors
                | Mode::BehaviorPreview
        ) && (146..183).contains(&y)
        {
            if self
                .runtime
                .app(self.selected)
                .is_some_and(|a| !a.spec.behaviors.is_empty())
            {
                return self.key(if x < 100 {
                    1003
                } else if x < 170 {
                    1007
                } else if x < 235 {
                    1013
                } else if x < 295 {
                    1014
                } else if x < 370 {
                    1011
                } else {
                    1012
                });
            }
            return self.key(if x < 110 {
                1003
            } else if x < 190 {
                1007
            } else if x < 280 {
                1013
            } else if x < 370 {
                1011
            } else {
                1012
            });
        }
        if self.mode == Mode::BehaviorPreview && self.behavior_is_all() && (617..649).contains(&y) {
            return self.key(if x < 240 { 1005 } else { 1006 });
        }
        if self.mode == Mode::Behaviors && (238..426).contains(&y) {
            let index = (y - 238) / 94;
            if index
                < self
                    .runtime
                    .app(self.selected)
                    .ok_or("app_not_found")?
                    .spec
                    .behaviors
                    .buttons()
                    .len()
            {
                self.focus = index;
                return self.key(13);
            }
        }
        if self.mode == Mode::Workflow && (238..574).contains(&y) {
            let index = (y - 238) / 84;
            if index
                < self
                    .runtime
                    .app(self.selected)
                    .ok_or("app_not_found")?
                    .spec
                    .workflow
                    .actions()
                    .len()
            {
                self.focus = index;
                return self.key(13);
            }
        }
        match self.mode {
            Mode::Home | Mode::Search => {
                if (146..195).contains(&y) {
                    self.mode = Mode::Search;
                    return Ok(Reply::State("search"));
                }
                for cell in 0..4 {
                    let left = 44 + (cell % 2) * 216;
                    let top = 242 + (cell / 2) * 178;
                    if (left..left + 176).contains(&x) && (top..top + 150).contains(&y) {
                        if let Some(slot) = (0..crate::microapp::APPS)
                            .filter(|i| self.app_matches(*i))
                            .nth(cell)
                        {
                            self.selected = slot;
                            self.form(None)?;
                        } else if cell
                            == self
                                .runtime
                                .entries()
                                .filter(|(i, _)| self.app_matches(*i))
                                .count()
                        {
                            self.begin_prompt(None);
                        }
                        return Ok(Reply::State(self.state()));
                    }
                }
            }
            Mode::Form => {
                if let Some(a) = self.runtime.app(self.selected)
                    && (190..622).contains(&y)
                {
                    let field = (y - 190) / 72;
                    if field < a.spec.fields().len()
                        && a.spec.fields()[field].kind != Kind::Calculated
                    {
                        self.focus = field;
                        if a.spec.fields()[field].kind == Kind::Check {
                            return self.key(32);
                        }
                    }
                }
            }
            Mode::Table => {
                if (244..564).contains(&y)
                    && let Some(index) = (0..8).filter(|r| self.visible(*r)).nth((y - 244) / 40)
                {
                    self.row = index;
                    return self.key(13);
                }
            }
            Mode::Records => {
                if (214..614).contains(&y) {
                    let row = (y - 214) / 50;
                    if let Some(index) = (0..8).filter(|r| self.visible(*r)).nth(row) {
                        self.row = index;
                        return self.key(13);
                    }
                }
            }
            _ => {}
        }
        Ok(Reply::State(self.state()))
    }
    fn begin_prompt(&mut self, target: Option<usize>) {
        self.target = target;
        self.prompt = Text::EMPTY;
        self.message = Text::EMPTY;
        self.mode = Mode::Compose;
    }
    fn form(&mut self, row: Option<usize>) -> Result<(), &'static str> {
        let a = self.runtime.app(self.selected).ok_or("app_not_found")?;
        self.draft = if let Some(r) = row {
            *a.row(r).ok_or("app_row_not_found")?
        } else {
            a.defaults()
        };
        self.columns = 0;
        self.editing = row;
        self.focus = a
            .spec
            .fields()
            .iter()
            .position(|f| f.kind != Kind::Calculated)
            .ok_or("app_input_required")?;
        self.message = Text::EMPTY;
        self.mode = Mode::Form;
        Ok(())
    }
    pub fn visible(&self, row: usize) -> bool {
        self.runtime.app(self.selected).is_some_and(|a| {
            a.row(row).is_some_and(|v| {
                !self.filtered || a.spec.check_field().is_none_or(|i| v[i].text() == "false")
            })
        })
    }
    fn normalize_row(&mut self) {
        if !self.visible(self.row) {
            self.row = (0..8).find(|r| self.visible(*r)).unwrap_or(0);
        }
    }
    fn run_control(&mut self) -> Result<Reply, &'static str> {
        if self.mode != Mode::Form {
            return Err("app_form_required");
        }
        let level = self.runtime.brightness(self.selected, &self.draft)?;
        if !self.display_available {
            return Err("display_unavailable");
        }
        self.brightness = level;
        self.message = Text::new("Brightness applied.")?;
        Ok(Reply::State("applied"))
    }
    fn key(&mut self, key: u32) -> Result<Reply, &'static str> {
        if key == 1010 {
            if !self.display_available {
                return Err("display_unavailable");
            }
            self.brightness = Brightness::FULL;
            self.message = Text::new("Display restored to 100%.")?;
            return Ok(Reply::State("display_reset"));
        }
        if key == 27 || key == 1001 {
            self.request = self.request.checked_add(1).ok_or("app_request_exhausted")?;
            self.mode = Mode::Home;
            self.message = Text::EMPTY;
            self.pending = Spec::EMPTY;
            self.search = Text::EMPTY;
            return Ok(Reply::State("home"));
        }
        if key == 1002
            && !matches!(
                self.mode,
                Mode::Thinking
                    | Mode::Review
                    | Mode::Transfer
                    | Mode::ImportReview
                    | Mode::WorkflowReview
                    | Mode::BehaviorReview
            )
        {
            let t = if matches!(
                self.mode,
                Mode::Form
                    | Mode::Records
                    | Mode::Table
                    | Mode::Workflow
                    | Mode::Behaviors
                    | Mode::BehaviorPreview
            ) {
                Some(self.selected)
            } else {
                None
            };
            self.begin_prompt(t);
            return Ok(Reply::State("compose"));
        }
        if matches!(
            self.mode,
            Mode::Form
                | Mode::Records
                | Mode::Table
                | Mode::Workflow
                | Mode::Behaviors
                | Mode::BehaviorPreview
        ) && self
            .runtime
            .app(self.selected)
            .is_some_and(|a| a.spec.capability() == Capability::Records)
        {
            match key {
                1014 => {
                    let a = self.runtime.app(self.selected).unwrap();
                    if a.spec.behaviors.is_empty() {
                        return Err("behavior_unavailable");
                    }
                    if self.mode == Mode::Form {
                        self.row = self.editing.ok_or("behavior_saved_row_required")?;
                    }
                    a.row(self.row).ok_or("app_row_not_found")?;
                    self.mode = Mode::Behaviors;
                    self.focus = 0;
                    self.message = Text::EMPTY;
                    self.behavior_error = None;
                    self.behavior_failed_row = 0;
                    return Ok(Reply::State("behaviors"));
                }
                1013 => {
                    let a = self.runtime.app(self.selected).unwrap();
                    if !a.spec.workflow.enabled() {
                        return Err("workflow_unavailable");
                    }
                    if self.mode == Mode::Form {
                        self.row = self.editing.ok_or("workflow_saved_row_required")?;
                    }
                    a.row(self.row).ok_or("app_row_not_found")?;
                    self.mode = Mode::Workflow;
                    self.focus = a
                        .spec
                        .workflow
                        .actions()
                        .iter()
                        .position(|act| Some(act.from) == a.state(self.row))
                        .unwrap_or(0);
                    self.message = Text::EMPTY;
                    return Ok(Reply::State("workflow"));
                }
                1011 | 1012 => {
                    self.request = self.request.checked_add(1).ok_or("app_request_exhausted")?;
                    self.base = self.runtime.revision();
                    self.importing = key == 1011;
                    self.import = ImportBatch::EMPTY;
                    self.message = Text::EMPTY;
                    self.mode = Mode::Transfer;
                    return Ok(Reply::Transfer {
                        request: self.request,
                        revision: self.base,
                        app: self.runtime.app(self.selected).unwrap().spec.id,
                        import: self.importing,
                    });
                }
                1007 => {
                    self.mode = Mode::Table;
                    self.normalize_row();
                    return Ok(Reply::State("table"));
                }
                1003 => {
                    self.mode = Mode::Records;
                    self.normalize_row();
                    return Ok(Reply::State("records"));
                }
                _ => {}
            }
        }
        match self.mode {
            Mode::Search => match key {
                8 | 127 => {
                    self.search.pop();
                    self.select_match();
                }
                32..=126 => {
                    self.search.push(key as u8);
                    self.select_match();
                }
                9 | 1006 => {
                    self.selected = (self.selected + 1..crate::microapp::APPS)
                        .chain(0..=self.selected)
                        .find(|i| self.app_matches(*i))
                        .unwrap_or(0);
                }
                13 if self.app_matches(self.selected) => self.form(None)?,
                13 => self.begin_prompt(None),
                _ => {}
            },
            Mode::Home => match key {
                47 => {
                    self.search = Text::EMPTY;
                    self.mode = Mode::Search;
                }
                9 | 1006 => {
                    let count = self.runtime.entries().count();
                    if count > 0 {
                        self.selected = (self.selected + 1) % count;
                    }
                }
                1005 => self.selected = self.selected.saturating_sub(1),
                13 => {
                    if self.runtime.app(self.selected).is_some() {
                        self.form(None)?;
                    } else {
                        self.begin_prompt(None);
                    }
                }
                32..=126 => {
                    self.begin_prompt(None);
                    self.prompt.push(key as u8);
                }
                _ => {}
            },
            Mode::Compose => match key {
                8 | 127 => self.prompt.pop(),
                32..=126 => self.prompt.push(key as u8),
                13 if !self.prompt.text().trim().is_empty() => {
                    self.request = self.request.checked_add(1).ok_or("app_request_exhausted")?;
                    self.base = self.runtime.revision();
                    self.pending = Spec::EMPTY;
                    self.mode = Mode::Thinking;
                    let target = self
                        .target
                        .and_then(|i| self.runtime.app(i))
                        .map_or(Text::EMPTY, |a| a.spec.id);
                    return Ok(Reply::Submit {
                        request: self.request,
                        revision: self.base,
                        target,
                        prompt: self.prompt,
                    });
                }
                _ => {}
            },
            Mode::Review if key == 13 && self.pending.workflow.enabled() => {
                self.mode = Mode::WorkflowReview;
            }
            Mode::Review | Mode::WorkflowReview if key == 13 => {
                if self.pending.behaviors.is_empty() {
                    self.install_pending()?;
                } else {
                    self.focus = 0;
                    self.mode = Mode::BehaviorReview;
                }
            }
            Mode::BehaviorReview if key == 13 => {
                if self.focus + 1 < self.pending.behaviors.buttons().len() {
                    self.focus += 1;
                } else {
                    self.install_pending()?;
                }
            }
            Mode::Form => {
                let a = self.runtime.app(self.selected).ok_or("app_not_found")?;
                if self
                    .editing
                    .and_then(|row| a.state(row))
                    .is_some_and(|state| a.spec.workflow.terminal(state))
                    && matches!(key, 8 | 13 | 32..=127)
                {
                    return Err("workflow_closed");
                }
                let kind = a.spec.fields()[self.focus].kind;
                match key {
                    9 | 1006 | 1005 => {
                        let len = a.spec.fields().len();
                        let step = if key == 1005 { len - 1 } else { 1 };
                        for _ in 0..len {
                            self.focus = (self.focus + step) % len;
                            if a.spec.fields()[self.focus].kind != Kind::Calculated {
                                break;
                            }
                        }
                    }
                    13 if a.spec.capability() == Capability::Brightness => {
                        return self.run_control();
                    }
                    13 => {
                        self.runtime
                            .save(self.selected, self.editing, &self.draft)?;
                        self.row = self
                            .editing
                            .unwrap_or(self.runtime.app(self.selected).unwrap().count() - 1);
                        self.mode = Mode::Records;
                        self.message = Text::new("Saved.")?;
                        self.normalize_row();
                    }
                    32 if kind == Kind::Check => {
                        self.draft[self.focus] =
                            Text::new(if self.draft[self.focus].text() == "true" {
                                "false"
                            } else {
                                "true"
                            })?
                    }
                    8 | 127 if matches!(kind, Kind::Text | Kind::Number) => {
                        self.draft[self.focus].pop()
                    }
                    32..=126 if matches!(kind, Kind::Text | Kind::Number) => {
                        self.draft[self.focus].push(key as u8)
                    }
                    _ => {}
                }
            }
            Mode::Records | Mode::Table => match key {
                1008 if self.mode == Mode::Table => {
                    if self
                        .runtime
                        .app(self.selected)
                        .is_some_and(|a| a.spec.fields().len() > 3)
                    {
                        self.columns = 1 - self.columns;
                    }
                }
                1009 if self.mode == Mode::Table => self.columns = 0,
                110 | 78 => self.form(None)?,
                13 if self.visible(self.row) => self.form(Some(self.row))?,
                32 if self.visible(self.row) => {
                    self.runtime.toggle(self.selected, self.row)?;
                    self.normalize_row();
                }
                1004 if self
                    .runtime
                    .app(self.selected)
                    .is_some_and(|a| a.spec.check_field().is_some()) =>
                {
                    self.filtered = !self.filtered;
                    self.normalize_row();
                }
                9 | 1006 => {
                    self.row = (self.row + 1..8)
                        .find(|r| self.visible(*r))
                        .or_else(|| (0..8).find(|r| self.visible(*r)))
                        .unwrap_or(0);
                }
                1005 => {
                    self.row = (0..self.row)
                        .rev()
                        .find(|r| self.visible(*r))
                        .unwrap_or(self.row);
                }
                _ => {}
            },
            Mode::ImportReview if key == 13 => {
                self.runtime
                    .append(self.selected, self.base, &self.import)?;
                self.import = ImportBatch::EMPTY;
                self.mode = Mode::Table;
                self.filtered = false;
                self.columns = 0;
                self.row = 0;
                self.message = Text::new("Imported. Existing rows preserved.")?;
            }
            Mode::ImportReview => {
                if matches!(key, 9 | 1006) {
                    self.row = (self.row + 1) % self.import.len();
                }
                if key == 1005 {
                    self.row = self.row.saturating_sub(1);
                }
            }
            Mode::Workflow => {
                let a = self.runtime.app(self.selected).ok_or("app_not_found")?;
                match key {
                    9 | 1006 => self.focus = (self.focus + 1) % a.spec.workflow.actions().len(),
                    1005 => self.focus = self.focus.saturating_sub(1),
                    13 => {
                        let id = a.spec.workflow.actions()[self.focus].id;
                        self.run_action(id.text(), self.runtime.revision())?;
                    }
                    _ => {}
                }
            }
            Mode::Behaviors => {
                let a = self.runtime.app(self.selected).ok_or("app_not_found")?;
                match key {
                    9 | 1006 => self.focus = (self.focus + 1) % a.spec.behaviors.buttons().len(),
                    1005 => self.focus = self.focus.saturating_sub(1),
                    13 => {
                        let id = a.spec.behaviors.buttons()[self.focus].id;
                        self.preview_behavior(id.text(), self.runtime.revision())?;
                    }
                    _ => {}
                }
            }
            Mode::BehaviorPreview if self.behavior_is_all() && matches!(key, 1005 | 1006 | 9) => {
                let count = self
                    .runtime
                    .app(self.selected)
                    .ok_or("app_not_found")?
                    .count();
                let row = if key == 1005 {
                    self.row.saturating_sub(1)
                } else {
                    (self.row + 1).min(count - 1)
                };
                let values = self
                    .runtime
                    .preview_behavior(
                        self.selected,
                        row,
                        self.behavior_id.text(),
                        self.behavior_base,
                    )
                    .map_err(|e| e.code)?;
                self.row = row;
                self.draft = values;
            }
            Mode::BehaviorPreview if key == 13 => {
                let id = self.behavior_id;
                self.run_behavior(id.text(), self.behavior_base)?;
            }
            Mode::Notice if key == 13 => self.mode = Mode::Home,
            _ => {}
        }
        Ok(Reply::State(self.state()))
    }
    fn install_pending(&mut self) -> Result<(), &'static str> {
        self.selected = self
            .runtime
            .install(&self.pending, self.base, self.target)?;
        self.filtered = false;
        self.row = 0;
        self.form(None)?;
        self.message = Text::new("App installed.")?;
        Ok(())
    }
    fn preview_behavior(&mut self, id: &str, base: u32) -> Result<(), &'static str> {
        self.behavior_error = None;
        self.behavior_failed_row = 0;
        if self.mode != Mode::Behaviors {
            self.behavior_error = Some(BehaviorError::admission("behavior_view_required"));
            return Err("behavior_view_required");
        }
        self.behavior_failed_row = 0;
        self.behavior_changed = 0;
        let all = self.runtime.app(self.selected).is_some_and(|a| {
            a.spec
                .behaviors
                .buttons()
                .iter()
                .any(|b| b.id.text() == id && b.all_rows)
        });
        if all {
            match self.runtime.preview_behavior_all(self.selected, id, base) {
                Ok(changed) => {
                    self.behavior_changed = changed;
                    self.row = 0;
                }
                Err(failed) => {
                    self.behavior_failed_row = failed.row;
                    self.behavior_error = Some(failed.error);
                    return Err(failed.error.code);
                }
            }
        }
        match self
            .runtime
            .preview_behavior(self.selected, self.row, id, base)
        {
            Ok(values) => {
                self.draft = values;
                self.behavior_id = Text::new(id)?;
                self.behavior_base = base;
                self.mode = Mode::BehaviorPreview;
                self.message = Text::EMPTY;
                Ok(())
            }
            Err(error) => {
                self.behavior_error = Some(error);
                Err(error.code)
            }
        }
    }
    fn run_behavior(&mut self, id: &str, base: u32) -> Result<(), &'static str> {
        self.behavior_error = None;
        self.behavior_failed_row = 0;
        if self.mode != Mode::BehaviorPreview
            || self.behavior_id.text() != id
            || self.behavior_base != base
        {
            self.behavior_error = Some(BehaviorError::admission("behavior_preview_required"));
            return Err("behavior_preview_required");
        }
        self.behavior_failed_row = 0;
        let result = if self.behavior_is_all() {
            self.runtime
                .run_behavior_all(self.selected, id, base)
                .map_err(|failed| {
                    self.behavior_failed_row = failed.row;
                    failed.error
                })
        } else {
            self.runtime.run_behavior(self.selected, self.row, id, base)
        };
        match result {
            Ok(()) => {
                self.mode = Mode::Behaviors;
                self.message = Text::new("All changes applied locally.")?;
                Ok(())
            }
            Err(error) => {
                self.behavior_error = Some(error);
                Err(error.code)
            }
        }
    }
    fn check_request(&self, id: &str) -> Result<(), &'static str> {
        if id.parse::<u32>().ok() != Some(self.request) || self.mode != Mode::Thinking {
            Err("app_stale_request")
        } else {
            Ok(())
        }
    }
    fn run_action(&mut self, id: &str, base: u32) -> Result<(), &'static str> {
        if self.mode != Mode::Workflow {
            return Err("workflow_view_required");
        }
        self.runtime
            .apply_action(self.selected, self.row, id, base)?;
        let a = self.runtime.app(self.selected).unwrap();
        self.focus = a
            .spec
            .workflow
            .actions()
            .iter()
            .position(|act| Some(act.from) == a.state(self.row))
            .unwrap_or(0);
        self.message = Text::new("Action applied locally.")?;
        Ok(())
    }
    fn check_transfer(&self, id: &str, import: bool) -> Result<(), &'static str> {
        if id.parse::<u32>().ok() != Some(self.request)
            || self.mode != Mode::Transfer
            || (import && !self.importing)
        {
            return Err("app_stale_transfer");
        }
        if self.base != self.runtime.revision() {
            return Err("app_stale_revision");
        }
        Ok(())
    }
    fn command(&mut self, name: &str, arg: Option<&str>) -> Result<Reply, &'static str> {
        if name == "app.open" {
            self.request = self.request.checked_add(1).ok_or("app_request_exhausted")?;
            self.active = true;
            self.message = Text::EMPTY;
            if let Some(id) = arg {
                self.selected = self.runtime.find(id).ok_or("app_not_found")?;
                self.form(None)?;
            } else {
                self.mode = Mode::Home;
                self.search = Text::EMPTY;
            }
            return Ok(Reply::State(self.state()));
        }
        if !self.active {
            return Err("app_ui_not_open");
        }
        if name == "app.run" {
            if arg.is_some() {
                return Err("bad_args");
            }
            return self.run_control();
        }
        let arg = arg.ok_or("bad_args")?;
        match name {
            "app.restore.begin" => {
                if arg != "1" || self.restore_started || self.runtime.revision() != 1 {
                    return Err("app_restore_denied");
                }
                self.restore_started = true;
                self.restoring = true;
            }
            "app.restore.state" => {
                if !self.restoring {
                    return Err("app_restore_denied");
                }
                let [id, row, state] = parts(arg)?;
                let index = self.runtime.find(id).ok_or("app_not_found")?;
                self.runtime
                    .restore_state(index, row.parse().map_err(|_| "bad_args")?, state)?;
            }
            "app.restore.end" => {
                if arg != "1" || !self.restoring {
                    return Err("app_restore_denied");
                }
                self.restoring = false;
            }
            "app.button" => {
                let [request, id, label, event] = parts(arg)?;
                self.check_request(request)?;
                self.pending
                    .behaviors
                    .add(Button::new(id, decode::<22>(label)?.text(), event)?)?;
            }
            "app.button.scope" => {
                let [request, id, scope] = parts(arg)?;
                self.check_request(request)?;
                self.pending.behaviors.set_scope(id, scope)?;
            }
            "app.step" => {
                let [request, button, op, field, value, compare, skip] = parts(arg)?;
                self.check_request(request)?;
                self.pending.behaviors.add_step(
                    button,
                    Step::new(
                        op,
                        field,
                        decode::<64>(value)?.text(),
                        compare,
                        skip.parse().map_err(|_| "bad_args")?,
                    )?,
                )?;
            }
            "app.behavior.preview" => {
                let [base, id] = parts(arg)?;
                self.preview_behavior(id, base.parse().map_err(|_| "bad_args")?)?;
            }
            "app.behavior.run" => {
                let [base, id] = parts(arg)?;
                self.run_behavior(id, base.parse().map_err(|_| "bad_args")?)?;
            }
            "app.workflow.run" => {
                let [base, id] = parts(arg)?;
                self.run_action(id, base.parse().map_err(|_| "bad_args")?)?;
            }
            "app.flow" => {
                let [id, initial] = parts(arg)?;
                self.check_request(id)?;
                if self.pending.workflow.enabled() {
                    return Err("workflow_duplicate");
                }
                self.pending.workflow = Workflow::new(initial)?;
            }
            "app.action" => {
                let [id, action, label, from, to, guard, op, value] = parts(arg)?;
                self.check_request(id)?;
                self.pending.workflow.add(Action::new(
                    action,
                    decode::<22>(label)?.text(),
                    from,
                    to,
                    guard,
                    op,
                    value.parse().map_err(|_| "bad_args")?,
                )?)?;
            }
            "app.import.begin" => {
                let [id, count] = parts(arg)?;
                self.check_transfer(id, true)?;
                if self.import.len() != 0 {
                    return Err("app_import_started");
                }
                self.import = ImportBatch::new(count.parse().map_err(|_| "bad_args")?)?;
            }
            "app.import.cell" => {
                let [id, row, field, value] = parts(arg)?;
                self.check_transfer(id, true)?;
                self.import.set(
                    row.parse().map_err(|_| "bad_args")?,
                    field.parse().map_err(|_| "bad_args")?,
                    decode(value)?,
                )?;
            }
            "app.import.ready" => {
                self.check_transfer(arg, true)?;
                self.runtime
                    .validate_append(self.selected, self.base, &self.import)?;
                self.row = 0;
                self.mode = Mode::ImportReview;
            }
            "app.transfer.notice" => {
                let (id, message) = arg.split_once(':').ok_or("bad_args")?;
                self.check_transfer(id, false)?;
                self.message = Text::new(message)?;
                self.mode = Mode::Notice;
            }
            "app.storage" => {
                if !matches!(
                    arg,
                    "saved" | "restored" | "unsaved" | "save failed" | "RAM only"
                ) {
                    return Err("bad_args");
                }
                self.storage = Text::new(arg)?;
            }
            "app.value" => {
                if self.mode != Mode::Form {
                    return Err("app_form_required");
                }
                let [index, value] = parts(arg)?;
                let index = index.parse::<usize>().map_err(|_| "bad_args")?;
                let app = self.runtime.app(self.selected).ok_or("app_not_found")?;
                if index >= app.spec.fields().len() {
                    return Err("app_bad_field_index");
                }
                if app.spec.fields()[index].kind == Kind::Calculated {
                    return Err("app_read_only");
                }
                if self
                    .editing
                    .and_then(|row| app.state(row))
                    .is_some_and(|state| app.spec.workflow.terminal(state))
                {
                    return Err("workflow_closed");
                }
                self.draft[index] = decode(value)?;
            }
            "app.source" => {
                self.live = match arg {
                    "meta" => true,
                    "replay" => false,
                    _ => return Err("bad_args"),
                }
            }
            "app.key" => return self.key(arg.parse().map_err(|_| "bad_args")?),
            "app.tap" => {
                let [x, y] = parts(arg)?;
                return self.tap(
                    x.parse().map_err(|_| "bad_args")?,
                    y.parse().map_err(|_| "bad_args")?,
                );
            }
            "app.begin" => {
                let [id, base, app, title, cap] = parts(arg)?;
                self.check_request(id)?;
                if base.parse::<u32>().ok() != Some(self.base) {
                    return Err("app_stale_revision");
                }
                self.pending = Spec::new(app, decode::<28>(title)?.text(), cap)?;
            }
            "app.field" => {
                let [id, key, kind, required, label, default] = parts(arg)?;
                self.check_request(id)?;
                let required = match required {
                    "0" => false,
                    "1" => true,
                    _ => return Err("bad_args"),
                };
                self.pending.add(Field::new(
                    key,
                    decode::<22>(label)?.text(),
                    kind,
                    required,
                    decode::<32>(default)?.text(),
                )?)?;
            }
            "app.formula" => {
                let [id, key, expression] = parts(arg)?;
                self.check_request(id)?;
                self.pending
                    .formula(key, decode::<64>(expression)?.text())?;
            }
            "app.filter" => {
                let [id, key, label] = parts(arg)?;
                self.check_request(id)?;
                self.pending.filter(key, decode::<18>(label)?.text())?;
            }
            "app.ready" => {
                let (id, reason) = arg.split_once(':').ok_or("bad_args")?;
                self.check_request(id)?;
                self.runtime
                    .validate(&self.pending, self.base, self.target)?;
                self.message = Text::new(reason)?;
                self.mode = Mode::Review;
            }
            "app.progress" => {
                let [id, base, attempt] = parts(arg)?;
                self.check_request(id)?;
                let base = base.parse::<u32>().map_err(|_| "bad_args")?;
                if base != self.base || base != self.runtime.revision() {
                    return Err("app_stale_revision");
                }
                self.message = Text::new(match attempt {
                    "1" => "Attempt 1 of 3: creating a proposal.",
                    "2" => "Attempt 2 of 3: correcting expressions.",
                    "3" => "Attempt 3 of 3: final correction.",
                    _ => return Err("app_attempt_limit"),
                })?;
            }
            "app.notice" => {
                let (id, reason) = arg.split_once(':').ok_or("bad_args")?;
                self.check_request(id)?;
                self.message = Text::new(reason)?;
                self.mode = Mode::Notice;
            }
            _ => return Err("unknown_operation"),
        }
        Ok(Reply::State(self.state()))
    }
    pub fn execute(&mut self, name: &str, arg: Option<&str>) -> Result<Reply, &'static str> {
        let result = self.command(name, arg);
        if let Err(error) = result.as_ref() {
            self.message = Text::new(error).unwrap_or(Text::EMPTY);
        }
        result
    }
}

pub struct DisplayState<'a>(pub &'a Controller);
impl core::fmt::Display for DisplayState<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "available={}; brightness={}; mode=software_rgb",
            self.0.display_available,
            self.0.brightness.percent()
        )
    }
}

/// Read-only diagnostic for a failed composed operation; no app data or authority.
pub struct BehaviorStatus<'a>(pub &'a Controller);
impl core::fmt::Display for BehaviorStatus<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let error = self.0.behavior_error;
        write!(
            f,
            "step={}; error={}",
            error.map_or(0, |e| e.step),
            error.map_or("", |e| e.code)
        )?;
        if self.0.behavior_failed_row > 0 {
            write!(f, "; row={}", self.0.behavior_failed_row)?;
        }
        Ok(())
    }
}

/// Read-only admission metadata for the host planning loop; contains no records.
pub struct PlanningState<'a>(pub &'a Controller);
impl core::fmt::Display for PlanningState<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "request={}; base={}; revision={}; state={}",
            self.0.request,
            self.0.base,
            self.0.runtime.revision(),
            self.0.state()
        )
    }
}

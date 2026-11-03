//! Guest UI controller for a bounded interpreter. No model or transport dependency.
use crate::microapp::{Field, Kind, Runtime, Spec, Text, Values};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Home,
    Compose,
    Thinking,
    Review,
    Form,
    Records,
    Notice,
}
pub struct Controller {
    pub active: bool,
    pub live: bool,
    pub mode: Mode,
    pub runtime: Runtime,
    pub selected: usize,
    pub focus: usize,
    pub row: usize,
    pub filtered: bool,
    pub draft: Values,
    pub editing: Option<usize>,
    pub prompt: Text<128>,
    pub message: Text<160>,
    pub pending: Spec,
    request: u32,
    base: u32,
    target: Option<usize>,
}
pub enum Reply {
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
            filtered: false,
            draft: [Text::EMPTY; 6],
            editing: None,
            prompt: Text::EMPTY,
            message: Text::EMPTY,
            pending: Spec::EMPTY,
            request: 0,
            base: 0,
            target: None,
        }
    }
    #[cfg(feature = "guest-display")]
    pub fn target(&self) -> Option<usize> {
        self.target
    }
    fn state(&self) -> &'static str {
        match self.mode {
            Mode::Home => "home",
            Mode::Compose => "compose",
            Mode::Thinking => "thinking",
            Mode::Review => "review",
            Mode::Form => "form",
            Mode::Records => "records",
            Mode::Notice => "notice",
        }
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
        self.editing = row;
        self.focus = 0;
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
    fn key(&mut self, key: u32) -> Result<Reply, &'static str> {
        if key == 27 || key == 1001 {
            self.request = self.request.checked_add(1).ok_or("app_request_exhausted")?;
            self.mode = Mode::Home;
            self.message = Text::EMPTY;
            self.pending = Spec::EMPTY;
            return Ok(Reply::State("home"));
        }
        if key == 1002 && !matches!(self.mode, Mode::Thinking | Mode::Review) {
            let t = if matches!(self.mode, Mode::Form | Mode::Records) {
                Some(self.selected)
            } else {
                None
            };
            self.begin_prompt(t);
            return Ok(Reply::State("compose"));
        }
        match self.mode {
            Mode::Home => match key {
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
            Mode::Review if key == 13 => {
                self.selected = self
                    .runtime
                    .install(&self.pending, self.base, self.target)?;
                self.filtered = false;
                self.row = 0;
                self.form(None)?;
                self.message = Text::new("Installed. Runs locally; no AI per click.")?;
            }
            Mode::Form => {
                let a = self.runtime.app(self.selected).ok_or("app_not_found")?;
                let kind = a.spec.fields()[self.focus].kind;
                match key {
                    9 | 1006 => self.focus = (self.focus + 1) % a.spec.fields().len(),
                    1005 => self.focus = self.focus.saturating_sub(1),
                    1003 => {
                        self.mode = Mode::Records;
                        self.normalize_row();
                    }
                    13 => {
                        self.runtime
                            .save(self.selected, self.editing, &self.draft)?;
                        self.row = self
                            .editing
                            .unwrap_or(self.runtime.app(self.selected).unwrap().count() - 1);
                        self.mode = Mode::Records;
                        self.message = Text::new("Record saved in this app's private namespace.")?;
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
                    8 | 127 if kind != Kind::Check => self.draft[self.focus].pop(),
                    32..=126 if kind != Kind::Check => self.draft[self.focus].push(key as u8),
                    _ => {}
                }
            }
            Mode::Records => match key {
                110 | 78 => self.form(None)?,
                13 if self.visible(self.row) => self.form(Some(self.row))?,
                32 if self.visible(self.row) => {
                    self.runtime.toggle(self.selected, self.row)?;
                    self.normalize_row();
                }
                1004 => {
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
            Mode::Notice if key == 13 => self.mode = Mode::Home,
            _ => {}
        }
        Ok(Reply::State(self.state()))
    }
    fn check_request(&self, id: &str) -> Result<(), &'static str> {
        if id.parse::<u32>().ok() != Some(self.request) || self.mode != Mode::Thinking {
            Err("app_stale_request")
        } else {
            Ok(())
        }
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
            }
            return Ok(Reply::State(self.state()));
        }
        if !self.active {
            return Err("app_ui_not_open");
        }
        let arg = arg.ok_or("bad_args")?;
        match name {
            "app.source" => {
                self.live = match arg {
                    "meta" => true,
                    "replay" => false,
                    _ => return Err("bad_args"),
                }
            }
            "app.key" => return self.key(arg.parse().map_err(|_| "bad_args")?),
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

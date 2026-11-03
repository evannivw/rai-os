//! Guest-owned prompt, proposal review and explicit confirmation state.
use crate::ramfs::{Plan, Store};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Compose,
    Thinking,
    Review,
    Done,
    Notice,
    Error,
}
pub struct Workspace {
    pub active: bool,
    pub phase: Phase,
    pub page: usize,
    pub live: bool,
    prompt: [u8; 128],
    len: usize,
    reason: [u8; 160],
    reason_len: usize,
    request: u32,
    pub plan: Plan,
}
pub enum Reply {
    State(&'static str),
    Submit(u32, [u8; 128], usize),
}
impl core::fmt::Display for Reply {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::State(s) => write!(f, "state={s}"),
            Self::Submit(id, prompt, len) => {
                write!(f, "submit={id}; prompt=")?;
                for b in &prompt[..*len] {
                    write!(f, "{b:02x}")?;
                }
                Ok(())
            }
        }
    }
}
impl Workspace {
    pub const fn new() -> Self {
        Self {
            active: false,
            phase: Phase::Compose,
            page: 0,
            live: false,
            prompt: [0; 128],
            len: 0,
            reason: [0; 160],
            reason_len: 0,
            request: 0,
            plan: Plan::new(0),
        }
    }
    #[cfg(feature = "guest-display")]
    pub fn prompt(&self) -> &str {
        core::str::from_utf8(&self.prompt[..self.len]).unwrap_or("")
    }
    #[cfg(feature = "guest-display")]
    pub fn reason(&self) -> &str {
        core::str::from_utf8(&self.reason[..self.reason_len]).unwrap_or("")
    }
    fn message(&mut self, s: &str) -> Result<(), &'static str> {
        if s.len() > 160 || !s.bytes().all(|b| (32..=126).contains(&b)) {
            return Err("ui_bad_text");
        }
        self.reason[..s.len()].copy_from_slice(s.as_bytes());
        self.reason_len = s.len();
        Ok(())
    }
    fn current<'a>(&self, arg: &'a str) -> Result<&'a str, &'static str> {
        let (id, rest) = arg.split_once(':').ok_or("bad_args")?;
        if id.parse::<u32>().ok() != Some(self.request) || self.phase != Phase::Thinking {
            return Err("ui_stale_request");
        }
        Ok(rest)
    }
    pub fn execute(
        &mut self,
        name: &str,
        arg: Option<&str>,
        store: &mut Store,
    ) -> Result<Reply, &'static str> {
        if name == "ui.open" {
            if arg.is_some() {
                return Err("bad_args");
            }
            self.active = true;
            self.phase = Phase::Compose;
            self.len = 0;
            self.reason_len = 0;
            self.request = self.request.checked_add(1).ok_or("ui_exhausted")?;
            return Ok(Reply::State("compose"));
        }
        if !self.active {
            return Err("ui_not_open");
        }
        let arg = arg.ok_or("bad_args")?;
        match name {
            "ui.source" => {
                self.live = match arg {
                    "meta" => true,
                    "replay" => false,
                    _ => return Err("bad_args"),
                };
            }
            "ui.key" => {
                let key = arg.parse::<u32>().map_err(|_| "bad_args")?;
                if key == 9 {
                    self.page = (self.page + 1) % 2;
                    return Ok(Reply::State("page"));
                }
                if key == 27 {
                    self.request = self.request.checked_add(1).ok_or("ui_exhausted")?;
                    self.phase = Phase::Compose;
                    self.reason_len = 0;
                    self.plan = Plan::new(0);
                    return Ok(Reply::State("cancelled"));
                }
                match self.phase {
                    Phase::Compose => match key {
                        13 if self.len > 0 => {
                            self.request = self.request.checked_add(1).ok_or("ui_exhausted")?;
                            self.phase = Phase::Thinking;
                            self.reason_len = 0;
                            self.plan = Plan::new(0);
                            return Ok(Reply::Submit(self.request, self.prompt, self.len));
                        }
                        8 | 127 => self.len = self.len.saturating_sub(1),
                        32..=126 if self.len < 128 => {
                            self.prompt[self.len] = key as u8;
                            self.len += 1;
                        }
                        _ => {}
                    },
                    Phase::Review if key == 13 => {
                        if let Err(error) = store.apply_plan(&self.plan) {
                            self.phase = Phase::Error;
                            self.message(error)?;
                            return Err(error);
                        }
                        self.phase = Phase::Done;
                        self.message("All changes applied together. Your workspace is ready.")?;
                    }
                    Phase::Done | Phase::Notice | Phase::Error if key == 13 => {
                        self.phase = Phase::Compose;
                        self.len = 0;
                        self.reason_len = 0;
                    }
                    _ => {}
                }
            }
            "ui.plan" => {
                let g = self.current(arg)?.parse().map_err(|_| "bad_args")?;
                self.plan = Plan::new(g);
            }
            "ui.action" => {
                let action = self.current(arg)?;
                self.plan.add(action)?;
            }
            "ui.ready" => {
                let reason = self.current(arg)?;
                store.validate_plan(&self.plan)?;
                self.message(reason)?;
                self.phase = Phase::Review;
            }
            "ui.notice" | "ui.error" => {
                let reason = self.current(arg)?;
                self.message(reason)?;
                self.phase = if name == "ui.notice" {
                    Phase::Notice
                } else {
                    Phase::Error
                };
            }
            _ => return Err("unknown_operation"),
        }
        Ok(Reply::State(match self.phase {
            Phase::Compose => "compose",
            Phase::Thinking => "thinking",
            Phase::Review => "review",
            Phase::Done => "done",
            Phase::Notice => "notice",
            Phase::Error => "error",
        }))
    }
}

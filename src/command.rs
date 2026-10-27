//! Allowed operations, independent of UART and wire framing.
use core::fmt;

pub const HELP: &str = "help | info | uptime | echo <text>";
pub const INFO: &str =
    "RAI-OS 0.1.0; aarch64; QEMU virt-9.2; EL1; single CPU; no process isolation; AI not connected";

pub enum Operation<'a> {
    Help,
    Info,
    Uptime,
    Echo(&'a str),
}

pub fn parse_shell(line: &str) -> Result<Operation<'_>, &'static str> {
    match line {
        "help" => Ok(Operation::Help),
        "info" => Ok(Operation::Info),
        "uptime" => Ok(Operation::Uptime),
        "echo" => Ok(Operation::Echo("")),
        _ => line
            .strip_prefix("echo ")
            .map(Operation::Echo)
            .ok_or("unknown_command"),
    }
}

/// A measured snapshot supplied by the hardware adapter, not read by commands.
#[derive(Clone, Copy)]
pub struct Uptime {
    pub milliseconds: u64,
    pub counter_hz: u32,
}

pub enum Reply<'a> {
    Text(&'a str),
    Uptime(Uptime),
}

impl fmt::Display for Reply<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => f.write_str(text),
            Self::Uptime(value) => write!(
                f,
                "uptime_ms={}; counter_hz={}",
                value.milliseconds, value.counter_hz
            ),
        }
    }
}

pub fn execute(operation: Operation<'_>, uptime: Uptime) -> Reply<'_> {
    match operation {
        Operation::Help => Reply::Text(HELP),
        Operation::Info => Reply::Text(INFO),
        Operation::Uptime => Reply::Uptime(uptime),
        Operation::Echo(text) => Reply::Text(text),
    }
}

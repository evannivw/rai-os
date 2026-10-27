//! Allowed operations, independent of UART and wire framing.
use crate::memory::{Check as MemoryCheck, Memory, Snapshot as MemorySnapshot};
use crate::task::{Snapshot, Task};
use core::fmt;

pub const HELP: &str = "help | info | uptime | memory.info | memory.check | echo <text> | task.start <text> | task.status | task.cancel <id>";
pub const INFO: &str =
    "RAI-OS 0.1.0; aarch64; QEMU virt-9.2; EL1; single CPU; no process isolation; AI not connected";

pub enum Operation<'a> {
    Help,
    Info,
    Uptime,
    MemoryInfo,
    MemoryCheck,
    Echo(&'a str),
    TaskStart(&'a str),
    TaskStatus,
    TaskCancel(u32),
}

pub fn parse_shell(line: &str) -> Result<Operation<'_>, &'static str> {
    match line {
        "help" => Ok(Operation::Help),
        "info" => Ok(Operation::Info),
        "memory.info" => Ok(Operation::MemoryInfo),
        "memory.check" => Ok(Operation::MemoryCheck),
        "uptime" => Ok(Operation::Uptime),
        "task.status" => Ok(Operation::TaskStatus),
        "task.start" => Ok(Operation::TaskStart("")),
        _ if line.starts_with("task.start ") => Ok(Operation::TaskStart(&line[11..])),
        _ if line.starts_with("task.cancel ") => {
            Ok(Operation::TaskCancel(parse_task_id(&line[12..])?))
        }
        "echo" => Ok(Operation::Echo("")),
        _ => line
            .strip_prefix("echo ")
            .map(Operation::Echo)
            .ok_or("unknown_command"),
    }
}

pub fn parse_task_id(text: &str) -> Result<u32, &'static str> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("bad_args");
    }
    text.parse::<u32>()
        .ok()
        .filter(|id| *id != 0)
        .ok_or("bad_args")
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
    Task(Snapshot),
    Memory(MemorySnapshot),
    MemoryCheck(MemoryCheck),
}

impl fmt::Display for Reply<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory(snapshot) => write!(f, "{snapshot}"),
            Self::MemoryCheck(check) => write!(f, "{check}"),
            Self::Task(snapshot) => write!(f, "{snapshot}"),
            Self::Text(text) => f.write_str(text),
            Self::Uptime(value) => write!(
                f,
                "uptime_ms={}; counter_hz={}",
                value.milliseconds, value.counter_hz
            ),
        }
    }
}

pub fn execute<'a>(
    operation: Operation<'a>,
    uptime: Uptime,
    task: &mut Task,
    memory: &mut Memory,
) -> Result<Reply<'a>, &'static str> {
    Ok(match operation {
        Operation::MemoryInfo => Reply::Memory(memory.snapshot()),
        Operation::MemoryCheck => Reply::MemoryCheck(memory.check()?),
        Operation::Help => Reply::Text(HELP),
        Operation::Info => Reply::Text(INFO),
        Operation::Uptime => Reply::Uptime(uptime),
        Operation::Echo(text) => Reply::Text(text),
        Operation::TaskStart(text) => Reply::Task(task.start(text, uptime.milliseconds)?),
        Operation::TaskStatus => Reply::Task(task.snapshot()),
        Operation::TaskCancel(id) => Reply::Task(task.cancel(id)?),
    })
}

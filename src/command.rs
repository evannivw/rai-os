//! Allowed operations, independent of UART and wire framing.
use crate::memory::{Check as MemoryCheck, Memory, Snapshot as MemorySnapshot};
use crate::mmu::Snapshot as MmuSnapshot;
use crate::task::{Snapshot, Task};
use core::fmt;

pub const HELP: &str = "help | info | uptime | mmu.info | context.check | timer.check | schedule.check | schedule.isolate | schedule.run | schedule.fault | user.run | user.spin | memory.info | memory.check | echo <text> | task.start <text> | task.status | task.cancel <id>";
pub const INFO: &str =
    "RAI-OS 0.1.0; aarch64; QEMU virt-9.2; EL1; single CPU; no process isolation; AI not connected";

pub enum Operation<'a> {
    Help,
    Info,
    Uptime,
    MmuInfo,
    ContextCheck,
    TimerCheck,
    ScheduleIsolate,
    ScheduleCheck,
    ScheduleRun,
    ScheduleFault,
    UserRun,
    UserSpin,
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
        "timer.check" => Ok(Operation::TimerCheck),
        "schedule.isolate" => Ok(Operation::ScheduleIsolate),
        "schedule.check" => Ok(Operation::ScheduleCheck),
        "schedule.run" => Ok(Operation::ScheduleRun),
        "schedule.fault" => Ok(Operation::ScheduleFault),
        "user.run" => Ok(Operation::UserRun),
        "user.spin" => Ok(Operation::UserSpin),
        "context.check" => Ok(Operation::ContextCheck),
        "mmu.info" => Ok(Operation::MmuInfo),
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
    User(crate::user::Report),
    Timer(crate::timer::Report),
    Isolation(crate::schedule::IsolationReport),
    Schedule(crate::schedule::Report),
    ScheduleRun(crate::schedule::RunReport),
    Mmu(MmuSnapshot),
    Memory(MemorySnapshot),
    MemoryCheck(MemoryCheck),
}

impl fmt::Display for Reply<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScheduleRun(report) => write!(f, "{report}"),
            Self::Isolation(report) => write!(f, "{report}"),
            Self::Schedule(report) => write!(f, "{report}"),
            Self::Timer(report) => write!(f, "{report}"),
            Self::User(report) => write!(f, "{report}"),
            Self::Mmu(snapshot) => write!(f, "{snapshot}"),
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
    mmu: &crate::mmu::Mmu,
    timer: &mut crate::timer::Timer,
) -> Result<Reply<'a>, &'static str> {
    Ok(match operation {
        Operation::ScheduleRun => Reply::ScheduleRun(crate::schedule::run(
            timer,
            mmu,
            crate::schedule::RunMode::Finish,
        )?),
        Operation::ScheduleFault => Reply::ScheduleRun(crate::schedule::run(
            timer,
            mmu,
            crate::schedule::RunMode::Fault,
        )?),
        Operation::ScheduleIsolate => Reply::Isolation(crate::schedule::isolate(timer, mmu)?),
        Operation::ScheduleCheck => Reply::Schedule(crate::schedule::check(timer, mmu)?),
        Operation::TimerCheck => Reply::Timer(timer.check()?),
        Operation::UserRun => Reply::User(crate::user::run(timer, crate::user::Program::Hello)?),
        Operation::UserSpin => Reply::User(crate::user::run(timer, crate::user::Program::Spin)?),
        Operation::ContextCheck => Reply::Text(crate::context::check()?),
        Operation::MmuInfo => Reply::Mmu(mmu.snapshot()),
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

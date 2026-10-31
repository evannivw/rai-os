//! Versioned, bounded line protocol. Parsing does not execute operations.
use crate::command::Operation;

pub struct Request<'a> {
    pub id: &'a str,
    pub operation: Operation<'a>,
}

pub struct Error<'a> {
    pub id: &'a str,
    pub code: &'static str,
}

pub fn parse(line: &str) -> Result<Request<'_>, Error<'_>> {
    let mut fields = line.split('\t');
    let version = fields.next();
    let id = fields.next().unwrap_or("");
    let valid_id = !id.is_empty()
        && id.len() <= 10
        && id.bytes().all(|b| b.is_ascii_digit())
        && id.parse::<u32>().is_ok_and(|n| n != 0);
    let id = if valid_id { id } else { "0" };
    let error = |code| Error { id, code };
    if version != Some("@rai1") {
        return Err(error("bad_version"));
    }
    if !valid_id {
        return Err(error("bad_id"));
    }
    let operation = match fields.next() {
        Some("help") => Operation::Help,
        Some("info") => Operation::Info,
        Some("timer.check") => Operation::TimerCheck,
        Some("ipc.wait") => Operation::IpcWait,
        Some("coord.start") => match fields.next() {
            None => Operation::CoordStart,
            Some("index") => Operation::CoordIndexStart,
            _ => return Err(error("bad_args")),
        },
        Some("coord.result") => Operation::CoordResult,
        Some("coord.bundle") => Operation::CoordBundle,
        Some("coord.bundle.status") => Operation::CoordBundleStatus,
        Some("coord.goal") => Operation::CoordGoal(
            crate::bundle::Required::parse(fields.next().ok_or(error("bad_args"))?)
                .map_err(error)?,
        ),
        Some("coord.status") => Operation::CoordStatus,
        Some("coord.stop") => Operation::CoordStop,
        Some("coord.observe") => Operation::CoordObserve(
            crate::records::RecordId::parse(fields.next().ok_or(error("bad_args"))?)
                .map_err(error)?,
        ),
        Some("coord.propose") => Operation::CoordPropose(
            crate::command::parse_proposal(fields.next().ok_or(error("bad_args"))?)
                .map_err(error)?,
        ),
        Some("worker.start") => Operation::WorkerStart,
        Some("worker.status") => Operation::WorkerStatus,
        Some("worker.stop") => Operation::WorkerStop,
        Some("worker.count") => Operation::WorkerCount(
            crate::records::RecordId::parse(fields.next().ok_or(error("bad_args"))?)
                .map_err(error)?,
        ),
        Some("record.index") => Operation::RecordIndex(
            crate::records::RecordId::parse(fields.next().ok_or(error("bad_args"))?)
                .map_err(error)?,
        ),
        Some("record.count") => Operation::RecordCount(
            crate::records::RecordId::parse(fields.next().ok_or(error("bad_args"))?)
                .map_err(error)?,
        ),
        Some("ipc.check") => Operation::IpcCheck,
        Some("schedule.isolate") => Operation::ScheduleIsolate,
        Some("schedule.check") => Operation::ScheduleCheck,
        Some("schedule.run") => Operation::ScheduleRun,
        Some("schedule.fault") => Operation::ScheduleFault,
        Some("user.run") => Operation::UserRun,
        Some("user.spin") => Operation::UserSpin,
        Some("context.check") => Operation::ContextCheck,
        Some("mmu.info") => Operation::MmuInfo,
        Some("memory.info") => Operation::MemoryInfo,
        Some("memory.check") => Operation::MemoryCheck,
        Some("uptime") => Operation::Uptime,
        Some("task.status") => Operation::TaskStatus,
        Some("task.start") => Operation::TaskStart(fields.next().ok_or(error("bad_args"))?),
        Some("task.cancel") => Operation::TaskCancel(
            crate::command::parse_task_id(fields.next().ok_or(error("bad_args"))?)
                .map_err(error)?,
        ),
        Some("echo") => Operation::Echo(fields.next().ok_or(error("bad_args"))?),
        Some(_) => return Err(error("unknown_operation")),
        None => return Err(error("bad_frame")),
    };
    if fields.next().is_some() {
        return Err(error("bad_args"));
    }
    Ok(Request { id, operation })
}

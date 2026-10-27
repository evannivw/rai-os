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

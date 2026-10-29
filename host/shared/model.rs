//! Pure operations and protocol validation: no terminal or process access.
pub enum Operation<'a> {
    Help,
    Info,
    Uptime,
    MmuInfo,
    ContextCheck,
    TimerCheck,
    IpcWait,
    IpcCheck,
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

pub struct Reply {
    pub success: bool,
    pub payload: String,
}

pub fn parse_input(line: &str) -> Result<Operation<'_>, &'static str> {
    match line {
        "help" => Ok(Operation::Help),
        "info" => Ok(Operation::Info),
        "timer.check" => Ok(Operation::TimerCheck),
        "ipc.wait" => Ok(Operation::IpcWait),
        "ipc.check" => Ok(Operation::IpcCheck),
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
            let text = &line[12..];
            if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
                return Err("Invalid task ID");
            }
            text.parse::<u32>()
                .ok()
                .filter(|id| *id != 0)
                .map(Operation::TaskCancel)
                .ok_or("Invalid task ID")
        }
        "echo" => Ok(Operation::Echo("")),
        _ => line
            .strip_prefix("echo ")
            .map(Operation::Echo)
            .ok_or("Use help, info, uptime, mmu.info, context.check, timer.check, schedule.check, schedule.isolate, ipc.check, ipc.wait, schedule.run, schedule.fault, user.run, user.spin, memory.info, memory.check, echo, task.start, task.status, task.cancel, or exit"),
    }
}

pub fn encode(id: u32, operation: &Operation<'_>) -> Result<String, &'static str> {
    if id == 0 {
        return Err("ID zero is reserved for transport errors");
    }
    let body = match operation {
        Operation::Help => "help".to_owned(),
        Operation::Info => "info".to_owned(),
        Operation::TimerCheck => "timer.check".to_owned(),
        Operation::IpcWait => "ipc.wait".to_owned(),
        Operation::IpcCheck => "ipc.check".to_owned(),
        Operation::ScheduleIsolate => "schedule.isolate".to_owned(),
        Operation::ScheduleCheck => "schedule.check".to_owned(),
        Operation::ScheduleRun => "schedule.run".to_owned(),
        Operation::ScheduleFault => "schedule.fault".to_owned(),
        Operation::UserRun => "user.run".to_owned(),
        Operation::UserSpin => "user.spin".to_owned(),
        Operation::ContextCheck => "context.check".to_owned(),
        Operation::MmuInfo => "mmu.info".to_owned(),
        Operation::MemoryInfo => "memory.info".to_owned(),
        Operation::MemoryCheck => "memory.check".to_owned(),
        Operation::Uptime => "uptime".to_owned(),
        Operation::TaskStatus => "task.status".to_owned(),
        Operation::TaskCancel(id) => {
            if *id == 0 {
                return Err("Invalid task ID");
            }
            format!("task.cancel\t{id}")
        }
        Operation::TaskStart(text) => {
            if text.len() > 192 {
                return Err("Task text exceeds 192 bytes");
            }
            if !text.bytes().all(|b| b.is_ascii_graphic() || b == b' ') {
                return Err("Task text requires printable ASCII");
            }
            format!("task.start\t{text}")
        }
        Operation::Echo(text) => {
            if !text.bytes().all(|b| b.is_ascii_graphic() || b == b' ') {
                return Err("echo accepts printable ASCII, without TAB or line breaks");
            }
            format!("echo\t{text}")
        }
    };
    let frame = format!("@rai1\t{id}\t{body}");
    if frame.len() > 256 {
        return Err("Request exceeds the 256-byte UART limit");
    }
    Ok(frame + "\n")
}

pub fn decode(id: u32, line: &str) -> Result<Reply, &'static str> {
    let fields: Vec<_> = line.split('\t').collect();
    if fields.len() != 4 || fields[0] != "@rai1" {
        return Err("Unexpected response format");
    }
    if fields[1] != id.to_string() {
        return Err("Unexpected response ID; cannot match this request");
    }
    if !fields[3].bytes().all(|b| b.is_ascii_graphic() || b == b' ') {
        return Err("Response contains control characters or non-ASCII bytes");
    }
    let success = match fields[2] {
        "ok" => true,
        "err" => false,
        _ => return Err("Unknown response status"),
    };
    Ok(Reply {
        success,
        payload: fields[3].to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_operations_are_bounded_and_argument_free() {
        for (name, operation) in [
            ("timer.check", Operation::TimerCheck),
            ("ipc.wait", Operation::IpcWait),
            ("ipc.check", Operation::IpcCheck),
            ("schedule.isolate", Operation::ScheduleIsolate),
            ("schedule.check", Operation::ScheduleCheck),
            ("schedule.run", Operation::ScheduleRun),
            ("schedule.fault", Operation::ScheduleFault),
            ("user.run", Operation::UserRun),
            ("user.spin", Operation::UserSpin),
            ("context.check", Operation::ContextCheck),
            ("mmu.info", Operation::MmuInfo),
            ("memory.info", Operation::MemoryInfo),
            ("memory.check", Operation::MemoryCheck),
        ] {
            assert_eq!(
                encode(7, &operation).unwrap(),
                format!("@rai1\t7\t{name}\n")
            );
            assert!(parse_input(name).is_ok());
            assert!(parse_input(&format!("{name} extra")).is_err());
            assert!(parse_input(&format!("{name}\t")).is_err());
        }
        assert!(parse_input("memory.free 0x40080000").is_err());
    }

    #[test]
    fn rejects_injection_and_oversized_requests() {
        for text in ["a\n@rai1\t2\tinfo", "a\rb", "a\tb", "\x1b[2J", "ñ"] {
            assert!(encode(1, &Operation::Echo(text)).is_err());
        }
        let prefix = "@rai1\t1\techo\t";
        let payload = "x".repeat(256 - prefix.len());
        assert_eq!(encode(1, &Operation::Echo(&payload)).unwrap().len(), 257);
        assert!(encode(1, &Operation::Echo(&(payload + "x"))).is_err());
    }

    #[test]
    fn rejects_wrong_request_and_malformed_responses() {
        for frame in [
            "@rai1\t2\tok\tx",
            "@rai2\t1\tok\tx",
            "@rai1\t1\tmaybe\tx",
            "@rai1\t1\tok\tx\textra",
            "@rai1\t1\tok\t\x1b[2J",
        ] {
            assert!(decode(1, frame).is_err());
        }
        assert!(decode(1, "@rai1\t1\tok\t").unwrap().success);
        assert!(!decode(1, "@rai1\t1\terr\tbad_args").unwrap().success);
    }
}

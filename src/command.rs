//! Allowed operations, independent of UART and wire framing.
pub const HELP: &str = "help | info | echo <text>";
pub const INFO: &str =
    "RAI-OS 0.1.0; aarch64; QEMU virt-9.2; EL1; single CPU; no process isolation; AI not connected";

pub enum Operation<'a> {
    Help,
    Info,
    Echo(&'a str),
}

pub fn parse_shell(line: &str) -> Result<Operation<'_>, &'static str> {
    match line {
        "help" => Ok(Operation::Help),
        "info" => Ok(Operation::Info),
        "echo" => Ok(Operation::Echo("")),
        _ => line
            .strip_prefix("echo ")
            .map(Operation::Echo)
            .ok_or("unknown_command"),
    }
}

pub fn execute<'a>(operation: Operation<'a>) -> &'a str {
    match operation {
        Operation::Help => HELP,
        Operation::Info => INFO,
        Operation::Echo(text) => text,
    }
}

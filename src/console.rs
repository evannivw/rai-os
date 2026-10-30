//! UART line transport and response rendering; no heap allocation or global buffers.
use crate::{
    clock::Clock, command, memory::Memory, mmu::Mmu, protocol, task::Task, timer::Timer, uart,
};
use core::fmt::Display;

const CAPACITY: usize = 256;

fn response(id: &str, status: &str, payload: impl Display) {
    uart::write("@rai1\t");
    uart::write(id);
    uart::put(b'\t');
    uart::write(status);
    uart::put(b'\t');
    uart::write_display(payload);
    uart::write("\n");
}

fn dispatch(
    line: &str,
    clock: &Clock,
    task: &mut Task,
    memory: &mut Memory,
    mmu: &Mmu,
    timer: &mut Timer,
    worker: &mut crate::schedule::worker::Worker,
) {
    if line.starts_with('@') {
        match protocol::parse(line) {
            Ok(request) => {
                match command::execute(
                    request.operation,
                    clock.snapshot(),
                    task,
                    memory,
                    mmu,
                    timer,
                    worker,
                ) {
                    Ok(reply) => response(request.id, "ok", reply),
                    Err(code) => response(request.id, "err", code),
                }
            }
            Err(error) => response(error.id, "err", error.code),
        }
    } else if !line.is_empty() {
        match command::parse_shell(line)
            .and_then(|op| command::execute(op, clock.snapshot(), task, memory, mmu, timer, worker))
        {
            Ok(reply) => uart::write_display(reply),
            Err(code) => {
                uart::write("error: ");
                uart::write(code);
            }
        }
        uart::write("\n");
    }
}

pub fn run(clock: &Clock, memory: &mut Memory, mmu: &Mmu, timer: &mut Timer) -> ! {
    let mut task = Task::new();
    let mut worker = crate::schedule::worker::Worker::new();
    let mut buffer = [0u8; CAPACITY];
    let mut length = 0;
    let mut rejected = None;
    let mut machine = false;
    let mut after_cr = false;
    uart::write("Console ready: type help for commands\n");
    loop {
        // Every loop gets a bounded task step, even during a partial/invalid frame.
        task.step(clock.snapshot().milliseconds);
        let byte = match uart::try_get() {
            Ok(Some(byte)) => byte,
            Ok(None) => {
                core::hint::spin_loop();
                continue;
            }
            Err(()) => {
                rejected = Some("uart_error");
                continue;
            }
        };
        if after_cr && byte == b'\n' {
            after_cr = false;
            continue;
        }
        after_cr = byte == b'\r';
        if byte == b'\r' || byte == b'\n' {
            if !machine {
                uart::write("\n");
            }
            if let Some(code) = rejected {
                // Framing errors cannot reliably preserve a request ID.
                response("0", "err", code);
            } else {
                // Input was restricted to ASCII; keep a checked conversion anyway.
                match core::str::from_utf8(&buffer[..length]) {
                    Ok(line) => dispatch(line, clock, &mut task, memory, mmu, timer, &mut worker),
                    Err(_) => response("0", "err", "invalid_byte"),
                }
            }
            length = 0;
            rejected = None;
            machine = false;
            continue;
        }
        if rejected.is_some() {
            continue; // Discard the ENTIRE rejected line, never execute its tail.
        }
        if length == 0 {
            machine = byte == b'@';
        }
        if !machine && (byte == 8 || byte == 127) {
            if length > 0 {
                length -= 1;
                uart::write("\x08 \x08");
            }
            continue;
        }
        if !(byte.is_ascii_graphic() || byte == b' ' || (machine && byte == b'\t')) {
            rejected = Some("invalid_byte");
        } else if length == CAPACITY {
            rejected = Some("line_too_long");
        } else {
            buffer[length] = byte;
            length += 1;
            if !machine {
                uart::put(byte);
            }
        }
    }
}

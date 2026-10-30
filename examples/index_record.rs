//! Host export adapter: preserve guest-produced index bytes, never tokenize here.
#[path = "../host/shared/model.rs"]
mod model;
#[path = "../host/shared/qemu.rs"]
mod qemu;
use std::{fs::OpenOptions, io::Write, time::Instant};
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("Usage: index_record <kernel ELF> <record key> <new output file>".into());
    }
    let input = format!("record.index {}", args[1]);
    let operation = model::parse_input(&input)?;
    model::encode(1, &operation)?;
    let started = Instant::now();
    let mut guest = qemu::Connection::start(&args[0])?;
    let boot_ms = started.elapsed().as_millis();
    let requested = Instant::now();
    let reply = guest.request(1, &operation)?;
    let request_us = requested.elapsed().as_micros();
    drop(guest);
    if !reply.success {
        return Err(format!("Guest rejected index: {}", reply.payload));
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])
        .map_err(|e| e.to_string())?;
    writeln!(output, "{}", reply.payload).map_err(|e| e.to_string())?;
    eprintln!(
        "Exported guest output: {} (not independently verified yet)",
        args[2]
    );
    eprintln!(
        "Observed host wall time: boot_ms={boot_ms}; request_us={request_us} (includes UART; not CPU-only time)"
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Index export failed: {error}");
        std::process::exit(1);
    }
}

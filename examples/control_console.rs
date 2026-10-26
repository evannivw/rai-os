//! Host application: composition root, separate from the bare-metal kernel.
#[path = "control_console/model.rs"]
mod model;
#[path = "control_console/qemu.rs"]
mod qemu;
#[path = "control_console/view.rs"]
mod view;

use std::io::{self, BufRead, Write};

fn run() -> Result<(), String> {
    let kernel = std::env::args()
        .nth(1)
        .ok_or("Falta la ruta del kernel ELF")?;
    view::header();
    let mut connection = qemu::Connection::start(&kernel)?;
    view::connected();
    let mut id = 1u32;
    let initial = connection.request(id, &model::Operation::Info)?;
    view::reply(&initial);
    let input = io::stdin();
    let mut lines = input.lock().lines();
    loop {
        print!("rai-host> ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        let Some(line) = lines.next() else { break };
        let line = line.map_err(|e| e.to_string())?;
        if line == "exit" {
            break;
        }
        if line.is_empty() {
            continue;
        }
        let operation = match model::parse_input(&line) {
            Ok(operation) => operation,
            Err(error) => {
                view::input_error(error);
                continue;
            }
        };
        let next_id = id.checked_add(1).ok_or("Se agotaron los identificadores")?;
        // Validate before transport. Invalid input never reaches the guest UART.
        if let Err(error) = model::encode(next_id, &operation) {
            view::input_error(error);
            continue;
        }
        id = next_id;
        let reply = connection.request(id, &operation)?;
        view::reply(&reply);
    }
    drop(connection); // Terminate and reap our QEMU child, including on EOF.
    println!("Sesion cerrada; QEMU detenido.");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Conexion cerrada: {error}");
        std::process::exit(1);
    }
}

//! Adapter: child process + UART stdio. One outstanding request at a time.
use super::model::{self, Operation, Reply};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

pub struct Connection {
    child: Child,
    input: ChildStdin,
    lines: Receiver<Result<String, String>>,
}

impl Connection {
    pub fn start(kernel: &str) -> Result<Self, String> {
        let mut child = Command::new("sh")
            .arg("scripts/qemu.sh")
            .arg(kernel)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("No se pudo iniciar QEMU: {e}"))?;
        // Pipes are guaranteed because Stdio::piped was requested above.
        let input = child.stdin.take().expect("piped stdin");
        let output = child.stdout.take().expect("piped stdout");
        let (sender, lines) = mpsc::sync_channel(16);
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let failed = line.is_err();
                if sender.send(line.map_err(|e| e.to_string())).is_err() || failed {
                    break;
                }
            }
        });
        let connection = Self {
            child,
            input,
            lines,
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut boot_seen = false;
        loop {
            let line = connection.receive(deadline)?;
            if line == "RAI-OS: boot OK | AArch64 EL1 | UART PL011" {
                boot_seen = true;
            }
            if boot_seen && line == "Console ready: help | info | echo <text>" {
                return Ok(connection);
            }
        }
    }

    fn receive(&self, deadline: Instant) -> Result<String, String> {
        let timeout = deadline.saturating_duration_since(Instant::now());
        self.lines
            .recv_timeout(timeout)
            .map_err(|e| format!("Sin respuesta del kernel (timeout o desconexion): {e}"))?
    }

    pub fn request(&mut self, id: u32, operation: &Operation<'_>) -> Result<Reply, String> {
        let frame = model::encode(id, operation)?;
        self.input
            .write_all(frame.as_bytes())
            .map_err(|e| e.to_string())?;
        self.input.flush().map_err(|e| e.to_string())?;
        let line = self.receive(Instant::now() + Duration::from_secs(10))?;
        model::decode(id, &line).map_err(str::to_owned)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

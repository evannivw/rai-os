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
        Self::start_mode(kernel, false)
    }
    #[allow(dead_code)] // Only the Meta index bridge currently opts into native guest graphics.
    pub fn start_graphics(kernel: &str) -> Result<Self, String> {
        Self::start_mode(kernel, true)
    }
    fn start_mode(kernel: &str, graphics: bool) -> Result<Self, String> {
        let mut command = Command::new("sh");
        command
            .env_remove("MODEL_API_KEY")
            .arg("scripts/qemu.sh")
            .arg(kernel);
        if graphics {
            command.arg("--graphics");
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Could not start QEMU: {e}"))?;
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
        let mut display_seen = false;
        loop {
            let line = connection.receive(deadline)?;
            if line == "RAI-OS: boot OK | AArch64 EL1 | UART PL011" {
                boot_seen = true;
            }
            if boot_seen && line == "Display ready: 480x800 guest framebuffer" {
                display_seen = true;
            }
            if boot_seen && line == "Console ready: type help for commands" {
                if graphics && !display_seen {
                    return Err("guest_display_not_ready".into());
                }
                return Ok(connection);
            }
        }
    }

    fn receive(&self, deadline: Instant) -> Result<String, String> {
        let timeout = deadline.saturating_duration_since(Instant::now());
        self.lines
            .recv_timeout(timeout)
            .map_err(|e| format!("No kernel response (timeout or disconnection): {e}"))?
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

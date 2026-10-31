//! Scripted events, real guest output and independent checks; no model inference.
#[allow(dead_code)]
#[path = "../host/shared/model.rs"]
mod model;
#[path = "../host/shared/qemu.rs"]
mod qemu;
use model::Operation;
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::Command,
    thread,
    time::Duration,
};
struct Demo {
    connection: qemu::Connection,
    serial: u32,
    graphics: bool,
    automatic: bool,
}
fn field<'a>(text: &'a str, key: &str) -> Result<&'a str, String> {
    text.split("; ")
        .filter_map(|p| p.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("Missing guest field: {key}"))
}
fn number(text: &str, key: &str) -> Result<u32, String> {
    field(text, key)?
        .parse()
        .map_err(|_| format!("Invalid {key}"))
}
impl Demo {
    fn request(&mut self, op: Operation<'_>) -> Result<String, String> {
        let reply = self.reply(op)?;
        if !reply.success {
            return Err(format!("Guest rejected request: {}", reply.payload));
        }
        Ok(reply.payload)
    }
    fn reply(&mut self, op: Operation<'_>) -> Result<model::Reply, String> {
        self.serial = self.serial.checked_add(1).ok_or("Request ID exhausted")?;
        self.connection.request(self.serial, &op)
    }
    fn pause(&self, label: &str) -> Result<(), String> {
        println!("{label}");
        if self.graphics {
            if self.automatic {
                thread::sleep(Duration::from_millis(1500));
            } else {
                print!("Press Enter to continue... ");
                io::stdout().flush().map_err(|e| e.to_string())?;
                let mut input = String::new();
                if io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| e.to_string())?
                    == 0
                {
                    return Err("Input closed; ending walkthrough".into());
                }
            }
        }
        Ok(())
    }
    fn fill(&mut self, goal: &str) -> Result<(), String> {
        for key in field(goal, "missing")?.split(',').filter(|k| *k != "-") {
            let job = self.request(Operation::CoordObserve(key))?;
            self.request(Operation::CoordPropose(number(&job, "ticket")?, key))?;
        }
        Ok(())
    }
    fn prepare(&mut self) -> Result<(String, u32), String> {
        let data = self.request(Operation::PublicationPrepare)?;
        let status = self.request(Operation::PublicationStatus)?;
        Ok((data, number(&status, "candidate_ticket")?))
    }
    fn verify(
        output: &Path,
        name: &str,
        data: &str,
        keys: &str,
        generation: u32,
    ) -> Result<bool, String> {
        let path = output.join(name);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{data}").map_err(|e| e.to_string())?;
        drop(file);
        let result = Command::new("python3")
            .args(["-B", "scripts/check_bundle.py"])
            .arg(&path)
            .args([
                "--required",
                keys,
                "--goal-version",
                &generation.to_string(),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(result.success())
    }
    fn run(&mut self, output: &Path) -> Result<(), String> {
        println!("RAI-OS publication walkthrough: scripted changes, real QEMU execution.");
        println!(
            "No Meta calls or performance comparison. One verification failure is deliberately injected."
        );
        self.request(Operation::CoordIndexStart)?;
        self.request(Operation::PublicationStatus)?;
        self.pause("NOT STARTED / No result has been published.")?;
        let keys = "brief@1,empty@1,notes@1";
        let goal = self.request(Operation::CoordGoal(keys))?;
        self.pause("PREPARING / The first requirement is active.")?;
        self.fill(&goal)?;
        let (initial, token) = self.prepare()?;
        self.pause("VERIFYING / Complete indexes are not yet a published result.")?;
        if !Self::verify(
            output,
            "initial.json",
            &initial,
            keys,
            number(&goal, "goal_version")?,
        )? {
            self.request(Operation::PublicationReject(token))?;
            return Err("Initial independent check failed".into());
        }
        self.request(Operation::PublicationCommit(token))?;
        self.pause("CURRENT / The independently checked result is now published.")?;
        let keys = "brief@1,empty@1,notes@2";
        let updated = self.request(Operation::CoordGoal(keys))?;
        if field(&updated, "missing")? != "notes@2" || field(&updated, "ready_records")? != "2" {
            return Err("Unexpected revision reuse".into());
        }
        if self.request(Operation::PublicationRead)? != initial {
            return Err("Previous publication changed".into());
        }
        self.pause("UPDATING / Two indexes reused; the previous result remains available.")?;
        self.fill(&updated)?;
        let (candidate, token) = self.prepare()?;
        // Corrupt only a host checker input, never the guest artifact or source data.
        if Self::verify(
            output,
            "injected-invalid.json",
            "{}",
            keys,
            number(&updated, "goal_version")?,
        )? {
            return Err("Checker accepted deliberately invalid input".into());
        }
        self.request(Operation::PublicationReject(token))?;
        if self.request(Operation::PublicationRead)? != initial {
            return Err("Failed update replaced prior output".into());
        }
        self.pause("BLOCKED / Deliberate failed verification. Previous result is still readable.")?;
        let (retry, fresh) = self.prepare()?;
        if retry != candidate || fresh == token {
            return Err("Invalid retry state".into());
        }
        let late = self.reply(Operation::PublicationCommit(token))?;
        if late.success || late.payload != "publication_stale_candidate" {
            return Err("Late confirmation was not rejected".into());
        }
        self.pause(
            "VERIFYING / An outdated confirmation was rejected without replacing the result.",
        )?;
        self.request(Operation::PublicationStatus)?;
        if !Self::verify(
            output,
            "updated.json",
            &retry,
            keys,
            number(&updated, "goal_version")?,
        )? {
            self.request(Operation::PublicationReject(fresh))?;
            return Err("Replacement independent check failed".into());
        }
        self.request(Operation::PublicationCommit(fresh))?;
        if self.request(Operation::PublicationRead)? != retry {
            return Err("Committed bytes differ".into());
        }
        let jobs = self.request(Operation::CoordStatus)?;
        if field(&jobs, "completed")? != "4" {
            return Err("Unexpected repeated worker jobs".into());
        }
        self.pause(
            "CURRENT / Updated result verified. Four total jobs; only one for the revision change.",
        )?;
        println!("Artifacts: {}", output.display());
        Ok(())
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 2 || args[2..].iter().any(|a| a != "--headless" && a != "--auto") {
        return Err(
            "Usage: publication_demo <kernel> <new output directory> [--auto] [--headless]".into(),
        );
    }
    let graphics = !args[2..].iter().any(|a| a == "--headless");
    let automatic = args[2..].iter().any(|a| a == "--auto");
    let output = Path::new(&args[1]);
    fs::create_dir(output).map_err(|e| format!("Output directory must be new: {e}"))?;
    let connection = if graphics {
        qemu::Connection::start_graphics(&args[0])?
    } else {
        qemu::Connection::start(&args[0])?
    };
    let mut demo = Demo {
        connection,
        serial: 0,
        graphics,
        automatic,
    };
    let result = demo.run(output);
    let cleanup = demo.request(Operation::CoordStop);
    result?;
    cleanup?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Publication demo failed: {error}");
        std::process::exit(1);
    }
}

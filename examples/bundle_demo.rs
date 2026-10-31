//! Scripted host walkthrough. Requirements/readiness and result bytes come from RAI-OS.
#[allow(dead_code)]
#[path = "../host/shared/model.rs"]
mod model;
#[path = "../host/shared/qemu.rs"]
mod qemu;
use model::Operation;
use std::{fs, io::Write, path::Path, process::Command, thread, time::Duration};
struct Demo {
    connection: qemu::Connection,
    serial: u32,
    graphics: bool,
}
impl Demo {
    fn request(&mut self, operation: Operation<'_>) -> Result<String, String> {
        self.serial += 1;
        let reply = self.connection.request(self.serial, &operation)?;
        if !reply.success {
            return Err(format!("Guest rejected operation: {}", reply.payload));
        }
        Ok(reply.payload)
    }
    fn pause(&self) {
        if self.graphics {
            thread::sleep(Duration::from_millis(900));
        }
    }
    fn run(&mut self, output: &Path) -> Result<(), String> {
        println!("Scripted correctness walkthrough; no Meta inference or comparative benchmark.");
        self.request(Operation::CoordIndexStart)?;
        for (step, (keys, expected_version, expected_jobs)) in [
            ("brief@1,empty@1,notes@1", 1, 3),
            ("brief@1,checklist@1,empty@1,notes@1", 5, 4),
            ("brief@1,checklist@1,empty@1,notes@2", 7, 5),
        ]
        .into_iter()
        .enumerate()
        {
            let goal = self.request(Operation::CoordGoal(keys))?;
            println!("Goal {}: {goal}", step + 1);
            if field(&goal, "goal_version")? != expected_version.to_string() {
                return Err("Unexpected goal version in fresh scripted session".into());
            }
            self.pause();
            // No local cache decides what to do: use the guest's missing set.
            for key in field(&goal, "missing")?
                .split(',')
                .filter(|key| *key != "-")
            {
                let state = self.request(Operation::CoordObserve(key))?;
                let ticket = field(&state, "ticket")?
                    .parse::<u32>()
                    .map_err(|_| "Invalid ticket")?;
                self.pause();
                println!(
                    "Job: {}",
                    self.request(Operation::CoordPropose(ticket, key))?
                );
                self.pause();
            }
            let status = self.request(Operation::CoordBundleStatus)?;
            if field(&status, "state")? != "complete"
                || field(&status, "completed")? != expected_jobs.to_string()
            {
                return Err("Unexpected completion or repeated work".into());
            }
            let payload = self.request(Operation::CoordBundle)?;
            let path = output.join(format!("goal-{}.json", step + 1));
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|e| e.to_string())?;
            writeln!(file, "{payload}").map_err(|e| e.to_string())?;
            drop(file);
            let checked = Command::new("python3")
                .args(["-B", "scripts/check_bundle.py"])
                .arg(&path)
                .args([
                    "--required",
                    keys,
                    "--goal-version",
                    &expected_version.to_string(),
                ])
                .status()
                .map_err(|e| e.to_string())?;
            if !checked.success() {
                return Err("Independent artifact verification failed".into());
            }
            println!("Published: {} | {status}", path.display());
            self.pause();
        }
        println!(
            "Three complete sets, five worker jobs. Only new required revisions were indexed."
        );
        self.pause();
        Ok(())
    }
}
fn field<'a>(text: &'a str, key: &str) -> Result<&'a str, String> {
    text.split("; ")
        .filter_map(|part| part.split_once('='))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("Missing guest field: {key}"))
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(args.len() == 2 || (args.len() == 3 && args[2] == "--graphics")) {
        return Err("Usage: bundle_demo <kernel> <new output directory> [--graphics]".into());
    }
    let output = Path::new(&args[1]);
    fs::create_dir(output).map_err(|e| format!("Output directory must be new: {e}"))?;
    let graphics = args.len() == 3;
    let connection = if graphics {
        qemu::Connection::start_graphics(&args[0])?
    } else {
        qemu::Connection::start(&args[0])?
    };
    let mut demo = Demo {
        connection,
        serial: 0,
        graphics,
    };
    let result = demo.run(output);
    let cleanup = demo.request(Operation::CoordStop);
    result?;
    cleanup?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Bundle demo failed: {error}");
        std::process::exit(1);
    }
}

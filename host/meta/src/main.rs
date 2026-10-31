//! Single-proposal, host-coordinated and guest-owned Meta bridge modes.
mod config;
mod coordinator;
mod decision_source;
mod domain;
mod guest_bridge;
mod guest_index;
mod guest_report;
mod loop_runner;
#[allow(dead_code)] // Share the established protocol without adding a second UART adapter.
#[path = "../../shared/model.rs"]
mod model;
mod provider;
#[path = "../../shared/qemu.rs"]
mod qemu;
mod set_bridge;
mod set_report;
use serde_json::json;
use std::{path::Path, time::Instant};
fn run() -> Result<(), String> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Err(
            "usage: rai-meta <kernel> [--graphics] --guest-set <events.json> | [--graphics] --guest-set-replay <events.json> <proposals.json> | <kernel> <goal> | --replay <proposal.json> | --coordinate <events.json> | --coordinate-replay <events.json> <proposals.json> | [--graphics] --guest-index <events.json> | [--graphics] --guest-index-replay <events.json> <proposals.json> | --guest <events.json> | --guest-replay <events.json> <proposals.json>".into(),
        );
    }
    let graphics = args.get(1).is_some_and(|s| s == "--graphics");
    if graphics {
        args.remove(1);
    }
    if graphics
        && !args.get(1).is_some_and(|s| {
            matches!(
                s.as_str(),
                "--guest-index" | "--guest-index-replay" | "--guest-set" | "--guest-set-replay"
            )
        })
    {
        return Err("graphics_requires_guest_index_mode".into());
    }
    let kernel = &args[0];
    if args.len() == 3 && args[1] == "--guest-set" {
        return set_bridge::run(kernel, Path::new(&args[2]), None, graphics);
    }
    if args.len() == 4 && args[1] == "--guest-set-replay" {
        return set_bridge::run(
            kernel,
            Path::new(&args[2]),
            Some(Path::new(&args[3])),
            graphics,
        );
    }
    if args.len() == 3 && args[1] == "--guest-index" {
        return guest_bridge::run(
            kernel,
            Path::new(&args[2]),
            None,
            domain::Work::Index,
            graphics,
        );
    }
    if args.len() == 4 && args[1] == "--guest-index-replay" {
        return guest_bridge::run(
            kernel,
            Path::new(&args[2]),
            Some(Path::new(&args[3])),
            domain::Work::Index,
            graphics,
        );
    }
    if args.len() == 3 && args[1] == "--coordinate" {
        return loop_runner::run(kernel, Path::new(&args[2]), None);
    }
    if args.len() == 4 && args[1] == "--coordinate-replay" {
        return loop_runner::run(kernel, Path::new(&args[2]), Some(Path::new(&args[3])));
    }
    if args.len() == 3 && args[1] == "--guest" {
        return guest_bridge::run(
            kernel,
            Path::new(&args[2]),
            None,
            domain::Work::Count,
            false,
        );
    }
    if args.len() == 4 && args[1] == "--guest-replay" {
        return guest_bridge::run(
            kernel,
            Path::new(&args[2]),
            Some(Path::new(&args[3])),
            domain::Work::Count,
            false,
        );
    }
    // Validate CLI shape before reading a credential or performing network I/O.
    let proposal = if args.len() == 3 && args[1] == "--replay" {
        let p = domain::parse_proposal(&config::bounded_file(Path::new(&args[2]), 2048)?)?;
        println!(
            "{}",
            json!({"event":"proposal_source","source":"replay","model":null})
        );
        p
    } else if args.len() == 2 && !args[1].starts_with("--") {
        domain::validate_goal(&args[1])?;
        let key = config::load()?;
        println!(
            "{}",
            json!({"event":"proposal_source","source":"meta","model":provider::MODEL})
        );
        let answer = provider::propose(&key, &args[1])?;
        println!(
            "{}",
            json!({"event":"model_response","model":provider::MODEL,"latency_ms":answer.latency_ms,"usage":answer.usage})
        );
        answer.proposal
    } else {
        return Err("invalid_bridge_arguments".into());
    };
    // Validate before printing the proposal or starting QEMU. JSON output escapes
    // all control characters; no model text is interpreted as a shell command.
    domain::validate(&proposal)?;
    println!(
        "{}",
        json!({"event":"validated_proposal","proposal":proposal})
    );
    let result = domain::execute(&proposal, |action| {
        let domain::Action::Count { key, .. } = action else {
            return Err("no_guest_action".into());
        };
        let start = Instant::now();
        let mut connection = qemu::Connection::start(kernel)?;
        println!("{}", json!({"event":"guest_boot_confirmed"}));
        let reply = connection.request(1, &model::Operation::RecordCount(key))?;
        if !reply.success {
            return Err("guest_rejected_operation".into());
        }
        let observed = domain::observe(action, &reply.payload)?;
        println!(
            "{}",
            json!({"event":"guest_result","request_id":1,"observed":observed,"payload":reply.payload,"boot_and_operation_ms":start.elapsed().as_millis()})
        );
        Ok(())
    })?;
    println!(
        "{}",
        json!({"event":"finished","status":if result.is_some() {"observed"} else {"declined"}})
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{}", json!({"event":"error","error":error}));
        std::process::exit(1);
    }
}

//! Trusted host validation acknowledgment; the guest owns candidate/publication state.
use crate::{guest_bridge::Transport, model::Operation, set_report::Snapshot};
use serde_json::json;
use std::{
    collections::BTreeMap,
    io::Write,
    process::{Command, Stdio},
};
pub(super) fn fields(text: &str) -> Result<BTreeMap<&str, &str>, String> {
    let mut result = BTreeMap::new();
    for pair in text.split("; ") {
        let (key, value) = pair.split_once('=').ok_or("invalid_publication_status")?;
        if result.insert(key, value).is_some() {
            return Err("duplicate_publication_field".into());
        }
    }
    if result.len() != 6
        || [
            "target_version",
            "published_version",
            "candidate_ticket",
            "state",
            "availability",
            "reason",
        ]
        .iter()
        .any(|k| !result.contains_key(k))
    {
        return Err("invalid_publication_status".into());
    }
    for key in ["target_version", "published_version", "candidate_ticket"] {
        if result[key].is_empty()
            || !result[key].bytes().all(|b| b.is_ascii_digit())
            || result[key].parse::<u32>().is_err()
        {
            return Err("invalid_publication_number".into());
        }
    }
    Ok(result)
}
fn exchange(
    t: &mut Transport,
    op: Operation<'_>,
    name: &str,
    arg: Option<&str>,
) -> Result<String, String> {
    let reply = t.exchange(&op, name, arg)?;
    if !reply.success {
        return Err(reply.payload);
    }
    Ok(reply.payload)
}
pub struct Candidate {
    pub ticket: u32,
    pub payload: String,
}
pub fn prepare(t: &mut Transport, snapshot: &Snapshot) -> Result<Candidate, String> {
    let payload = exchange(
        t,
        Operation::PublicationPrepare,
        "coord.publish.prepare",
        None,
    )?;
    snapshot.check_artifact(&payload)?;
    let text = exchange(
        t,
        Operation::PublicationStatus,
        "coord.publish.status",
        None,
    )?;
    let status = fields(&text)?;
    let ticket = status["candidate_ticket"]
        .parse::<u32>()
        .map_err(|_| "invalid_publication_number")?;
    if status["target_version"] != snapshot.goal_version.to_string()
        || status["state"] != "validating"
        || ticket <= snapshot.goal_version
        || status["reason"] != "-"
    {
        return Err("unexpected_publication_candidate".into());
    }
    // Independent implementation checks exact fixture source bytes and token counts.
    // Neither a model answer nor the guest's structural check supplies this verdict.
    let mut checker = Command::new("python3")
        .args([
            "-B",
            "scripts/check_bundle.py",
            "-",
            "--required",
            &snapshot.required.join(","),
            "--goal-version",
            &snapshot.goal_version.to_string(),
        ])
        .env_remove("MODEL_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "cannot_start_bundle_checker")?;
    let write = checker
        .stdin
        .take()
        .ok_or("missing_checker_input")
        .and_then(|mut input| {
            writeln!(input, "{payload}").map_err(|_| "cannot_write_checker_input")
        });
    if write.is_err() {
        let _ = checker.kill();
    }
    let checked = checker.wait().map_err(|_| "cannot_wait_for_checker")?;
    let passed = write.is_ok() && checked.success();
    println!(
        "{}",
        json!({"event":"independent_check","goal_version":snapshot.goal_version,"candidate_ticket":ticket,"passed":passed})
    );
    if !passed {
        exchange(
            t,
            Operation::PublicationReject(ticket),
            "coord.publish.reject",
            Some(&ticket.to_string()),
        )?;
        return Err("independent_bundle_check_failed".into());
    }
    Ok(Candidate { ticket, payload })
}
pub fn commit(t: &mut Transport, snapshot: &Snapshot, candidate: Candidate) -> Result<(), String> {
    let text = exchange(
        t,
        Operation::PublicationCommit(candidate.ticket),
        "coord.publish.commit",
        Some(&candidate.ticket.to_string()),
    )?;
    let s = fields(&text)?;
    if s["target_version"] != snapshot.goal_version.to_string()
        || s["published_version"] != snapshot.goal_version.to_string()
        || s["candidate_ticket"] != "0"
        || s["state"] != "current"
        || s["availability"] != "current"
        || s["reason"] != "-"
    {
        return Err("unexpected_publication_commit".into());
    }
    let payload = exchange(t, Operation::PublicationRead, "coord.publish.read", None)?;
    if payload != candidate.payload {
        return Err("publication_bytes_changed".into());
    }
    println!(
        "{}",
        json!({"event":"batch_publication","snapshot":snapshot,"payload":payload})
    );
    Ok(())
}

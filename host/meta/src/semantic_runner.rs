//! Two explicit phases: propose a requirement, then execute an approved selection.
use crate::{batch_bridge, config, decision_source, provider, semantic};
use serde_json::json;
use std::{path::Path, time::Instant};
fn request(path: &Path) -> Result<semantic::Request, String> {
    Ok(semantic::Request::parse(&config::bounded_file(
        path, 4096,
    )?)?)
}
pub fn interpret(path: &Path, replay: Option<&Path>) -> Result<(), String> {
    let r = request(path)?;
    let started = Instant::now();
    println!(
        "{}",
        json!({"event":"semantic_started","source":if replay.is_some(){"replay"}else{"meta"},"request":r,"catalog_version":semantic::CATALOG_VERSION,"permissions_version":semantic::PERMISSIONS_VERSION})
    );
    let s = if let Some(path) = replay {
        semantic::parse(&config::bounded_file(path, 2048)?)?
    } else {
        let key = config::load()?;
        let answer =
            decision_source::report(provider::interpret(&key, &r), u64::from(r.request_version))?;
        println!(
            "{}",
            json!({"event":"semantic_response","latency_ms":answer.latency_ms,"usage":answer.usage})
        );
        answer.proposal
    };
    semantic::validate(&s, &r)?;
    println!(
        "{}",
        json!({"event":"semantic_selection","selection":s,"elapsed_ms":started.elapsed().as_millis(),"validation":"authority_only_not_intent_correctness"})
    );
    Ok(())
}
pub fn execute(kernel: &str, path: &Path, selection: &Path, graphics: bool) -> Result<(), String> {
    let r = request(path)?;
    let s = semantic::parse(&config::bounded_file(selection, 2048)?)?;
    let keys = semantic::validate(&s, &r)?;
    println!(
        "{}",
        json!({"event":"semantic_execution","request":r,"selection":s,"approval":"trusted_host_input","intent_verified_by_this_command":false})
    );
    if let Some(keys) = keys {
        batch_bridge::run_selected(kernel, &keys, graphics)
    } else {
        println!(
            "{}",
            json!({"event":"semantic_no_execution","decision":s.decision})
        );
        Ok(())
    }
}

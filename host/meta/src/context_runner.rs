//! Context-triggered evaluation: constant user objective, versioned observations.
use crate::{config, context, decision_source, provider, semantic};
use serde_json::json;
use std::{path::Path, time::Instant};
pub fn interpret(path: &Path, source: &str, replay: Option<&Path>) -> Result<(), String> {
    if !matches!(source, "rules" | "meta" | "replay") || (source == "replay") != replay.is_some() {
        return Err("invalid_context_source".into());
    }
    let observation = context::Observation::parse(&config::bounded_file(path, 4096)?)?;
    let request = observation.request();
    let started = Instant::now();
    println!(
        "{}",
        json!({"event":"semantic_started","source":source,"request":request,"observation":observation,"trigger":"controlled_context_update","user_goal_changed":false})
    );
    let selection = match source {
        "rules" => context::resolve(&observation),
        "replay" => semantic::parse(&config::bounded_file(
            replay.ok_or("missing_context_replay")?,
            2048,
        )?)?,
        _ => {
            let key = config::load()?;
            let answer = decision_source::report(
                provider::interpret_context(&key, &observation),
                u64::from(observation.version),
            )?;
            println!(
                "{}",
                json!({"event":"semantic_response","latency_ms":answer.latency_ms,"usage":answer.usage})
            );
            answer.proposal
        }
    };
    semantic::validate(&selection, &request)?;
    println!(
        "{}",
        json!({"event":"semantic_selection","selection":selection,"elapsed_ms":started.elapsed().as_millis(),"validation":"authority_only_not_intent_correctness"})
    );
    Ok(())
}

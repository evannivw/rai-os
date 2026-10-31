//! One common real-QEMU executor for fixed/model pilot policies. Budgets are host simulation.
use crate::{
    config,
    decision_source::{Pending, Source},
    domain::{self, DecisionContext, Proposal, Work},
    guest_bridge::Transport,
    guest_report,
    model::Operation,
    pilot::{self, Scenario},
    qemu,
    set_report::Snapshot,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
struct Run {
    t: Transport,
    scenario: Scenario,
    cursor: usize,
    event_cutoff: u64,
    set: Option<Snapshot>,
    ready: BTreeMap<String, u64>,
    spent: u64,
    decisions: usize,
    stale: usize,
    published: Option<(u32, u64)>,
    pending: Option<(Pending, DecisionContext)>,
}
impl Run {
    fn now(&self) -> u64 {
        self.t.started.elapsed().as_millis() as u64
    }
    fn due(&mut self) -> Result<(), String> {
        self.event_cutoff = self.now();
        while let Some(e) = self.scenario.events.get(self.cursor) {
            if e.at_ms > self.event_cutoff {
                break;
            }
            let required = crate::set_report::keys(e.jobs.iter().map(|j| j.record.as_str()))?;
            let argument = required.join(",");
            println!(
                "{}",
                json!({"event":"pilot_event","index":self.cursor,"at_ms":self.now(),"input":e})
            );
            let reply = self.t.exchange(
                &Operation::CoordGoal(&argument),
                "coord.goal",
                Some(&argument),
            )?;
            if !reply.success {
                return Err(reply.payload);
            }
            let next = Snapshot::parse(&reply.payload)?;
            if next.required != required
                || next.completed != self.ready.len() as u64
                || self
                    .set
                    .as_ref()
                    .is_some_and(|old| next.goal_version <= old.goal_version)
                || next.missing
                    != required
                        .iter()
                        .copied()
                        .filter(|k| !self.ready.contains_key(*k))
                        .collect::<Vec<_>>()
            {
                return Err("unexpected_pilot_goal".into());
            }
            self.set = Some(next);
            self.cursor += 1;
            self.published = None;
        }
        Ok(())
    }
    fn status(&mut self) -> Result<Snapshot, String> {
        let r = self
            .t
            .exchange(&Operation::CoordBundleStatus, "coord.bundle.status", None)?;
        if !r.success {
            return Err(r.payload);
        }
        Ok(Snapshot::parse(&r.payload)?)
    }
    fn context(&self) -> Result<DecisionContext, String> {
        Ok(pilot::context(
            self.set.as_ref().ok_or("missing_pilot_goal")?,
            &self.scenario.events[self.cursor - 1],
            &self.ready,
            self.spent,
            self.now(),
            self.scenario.deadline_ms,
        )?)
    }
    fn execute(&mut self, proposal: &Proposal, original: &DecisionContext) -> Result<(), String> {
        let action = domain::validate_for(proposal, original.state_version, Work::Index)?;
        self.due()?;
        let old = self.set.clone().ok_or("missing_pilot_goal")?;
        if matches!(action, domain::Action::Decline) {
            println!(
                "{}",
                json!({"event":"pilot_declined","proposal":proposal,"obsolete":original.state_version!=u64::from(old.goal_version)})
            );
            if original.state_version == u64::from(old.goal_version) {
                return Err("policy_declined".into());
            }
            return Ok(());
        }
        let domain::Action::Index { key, .. } = action else {
            return Err("operation_not_allowed".into());
        };
        if !original.authorized_records.iter().any(|(k, _)| *k == key) {
            return Err("proposal_not_authorized_at_request".into());
        }
        if self.now() >= self.scenario.deadline_ms {
            return Err("deadline_exceeded".into());
        }
        let generation =
            u32::try_from(original.state_version).map_err(|_| "invalid_goal_version")?;
        let argument = format!("{generation}:{key}");
        println!(
            "{}",
            json!({"event":"pilot_selection","at_ms":self.now(),"proposal":proposal})
        );
        let reply = self.t.exchange(
            &Operation::CoordSelect(generation, key),
            "coord.select",
            Some(&argument),
        )?;
        if generation != old.goal_version {
            if reply.success || reply.payload != "coord_stale_goal" || self.status()? != old {
                return Err("unexpected_stale_goal_result".into());
            }
            self.stale += 1;
            return Ok(());
        }
        if !reply.success {
            return Err(reply.payload);
        }
        let selected = guest_report::Snapshot::parse(&reply.payload)?;
        if selected.state != "awaiting_proposal"
            || selected.event != "observed"
            || selected.record != Some(key)
            || selected.completed != old.completed
            || selected.ticket <= generation
            || selected.b_calls != 0
        {
            return Err("unexpected_pilot_selection".into());
        }
        self.due()?;
        if self.set.as_ref().unwrap().goal_version != generation {
            self.stale += 1;
            println!(
                "{}",
                json!({"event":"pilot_selection_superseded","ticket":selected.ticket,"at_ms":self.now(),"event_cutoff_ms":self.event_cutoff})
            );
            return Ok(());
        }
        if self.now() >= self.scenario.deadline_ms {
            return Err("deadline_exceeded".into());
        }
        let cost = pilot::credits(key)?;
        let cap = self.scenario.events[self.cursor - 1].credit_cap;
        if self.spent + cost > cap {
            return Err("credit_limit".into());
        }
        self.spent += cost;
        println!(
            "{}",
            json!({"event":"pilot_debit","record":key,"credits":cost,"spent":self.spent,"cap":cap,"goal_version":generation,"ticket":selected.ticket,"at_ms":self.now(),"event_cutoff_ms":self.event_cutoff})
        );
        let argument = format!("{}:{key}", selected.ticket);
        let next = self.t.snapshot(
            &Operation::CoordPropose(selected.ticket, key),
            "coord.propose",
            Some(&argument),
        )?;
        next.check_completion(&selected, &action, selected.ticket)?;
        self.t.artifact(&next)?;
        let at = self.now();
        if self.ready.insert(key.to_string(), at).is_some() {
            return Err("pilot_repeated_work".into());
        }
        println!("{}", json!({"event":"pilot_ready","record":key,"at_ms":at}));
        let after = self.status()?;
        if after.goal_version != generation
            || after.required != old.required
            || after.completed != self.ready.len() as u64
            || after.missing
                != old
                    .missing
                    .iter()
                    .copied()
                    .filter(|k| *k != key)
                    .collect::<Vec<_>>()
        {
            return Err("unexpected_pilot_progress".into());
        }
        self.set = Some(after);
        Ok(())
    }
    fn publish(&mut self) -> Result<(), String> {
        let set = self.set.clone().ok_or("missing_pilot_goal")?;
        let r = self
            .t
            .exchange(&Operation::CoordBundle, "coord.bundle", None)?;
        if !r.success {
            return Err(r.payload);
        }
        set.check_artifact(&r.payload)?;
        // Same independent source/index verifier for every policy; its real wall
        // time is included before marking a publication as verified.
        let mut child = Command::new("python3")
            .args(["-B", "scripts/verify_pilot.py", "--artifact"])
            .env_remove("MODEL_API_KEY")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "cannot_start_artifact_checker")?;
        let input =
            json!({"payload":r.payload,"required":set.required,"goal_version":set.goal_version})
                .to_string();
        let written = child
            .stdin
            .take()
            .ok_or("missing_checker_input")?
            .write_all(input.as_bytes());
        let output = child
            .wait_with_output()
            .map_err(|_| "cannot_wait_for_artifact_checker")?;
        if written.is_err() || !output.status.success() || output.stdout != b"VERIFIED\n" {
            return Err("independent_artifact_rejected".into());
        }
        let at = self.now();
        self.published = Some((set.goal_version, at));
        println!(
            "{}",
            json!({"event":"pilot_verified_bundle","snapshot":set,"payload":r.payload,"at_ms":at})
        );
        Ok(())
    }
    fn session(&mut self, policy: &str, source: &mut Option<Source>) -> Result<(), String> {
        let start = self
            .t
            .snapshot(&Operation::CoordIndexStart, "coord.start", Some("index"))?;
        if start.state != "idle" || start.completed != 0 {
            return Err("guest_not_fresh".into());
        }
        let mut blocked = None;
        loop {
            self.due()?;
            if self.now() >= self.scenario.deadline_ms {
                return Err("deadline_exceeded".into());
            }
            if let Some((pending, context)) = &self.pending
                && let Some(answer) = pending.poll()?
            {
                let original = context.clone();
                println!(
                    "{}",
                    json!({"event":"pilot_decision_response","call_id":self.decisions,"latency_ms":answer.latency_ms,"usage":answer.usage,"proposal":answer.proposal,"at_ms":self.now()})
                );
                drop(self.pending.take());
                self.execute(&answer.proposal, &original)?;
            }
            self.due()?;
            if self.now() >= self.scenario.deadline_ms {
                return Err("deadline_exceeded".into());
            }
            let current = self.set.as_ref().ok_or("missing_pilot_goal")?;
            if current.missing.is_empty() && self.published.is_none() {
                self.publish()?;
            }
            // A result for a superseded generation is never the final score.
            self.due()?;
            let current = self.set.as_ref().unwrap();
            if self.cursor == self.scenario.events.len() && current.missing.is_empty() {
                let (version, at) = self.published.ok_or("missing_verified_publication")?;
                if version != current.goal_version || at > self.scenario.deadline_ms {
                    return Err("deadline_exceeded".into());
                }
                if source.as_ref().is_some_and(Source::has_unused) {
                    return Err("unused_replay_proposals".into());
                }
                return Ok(());
            }
            if self.pending.is_none() && !current.missing.is_empty() {
                let context = self.context()?;
                if context.authorized_records.is_empty() {
                    if blocked != Some(current.goal_version) {
                        println!(
                            "{}",
                            json!({"event":"pilot_blocked","reason":"no_affordable_work","at_ms":self.now(),"snapshot":context})
                        );
                        blocked = Some(current.goal_version);
                    }
                } else {
                    if self.decisions == pilot::MAX_DECISIONS {
                        return Err("decision_budget_exhausted".into());
                    }
                    self.decisions += 1;
                    println!(
                        "{}",
                        json!({"event":"pilot_decision_requested","call_id":self.decisions,"snapshot":context,"at_ms":self.now()})
                    );
                    if let Some(source) = source {
                        self.pending = Some((source.start(pilot::GOAL, context.clone())?, context));
                    } else {
                        let start = Instant::now();
                        let key = pilot::choose(policy, &context)?;
                        let proposal = Proposal {
                            state_version: context.state_version,
                            operation: "record.index".into(),
                            record: key.into(),
                            reason: format!("Fixed {policy} rule on the shared current snapshot."),
                        };
                        println!(
                            "{}",
                            json!({"event":"pilot_decision_response","call_id":self.decisions,"latency_ms":start.elapsed().as_millis(),"usage":null,"proposal":proposal,"at_ms":self.now()})
                        );
                        self.execute(&proposal, &context)?;
                    }
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}
pub fn run(kernel: &str, policy: &str, path: &Path, replay: Option<&Path>) -> Result<(), String> {
    if !matches!(policy, "edf" | "value" | "meta" | "replay")
        || (policy == "replay") != replay.is_some()
    {
        return Err("invalid_pilot_arguments".into());
    }
    let scenario = Scenario::parse(&config::bounded_file(path, 8192)?)?;
    let mut source = if matches!(policy, "meta" | "replay") {
        Some(Source::load(replay, pilot::MAX_DECISIONS)?)
    } else {
        None
    };
    println!(
        "{}",
        json!({"event":"pilot_started","scenario_id":scenario.id,"policy":policy,"model":if policy=="meta"{Some(crate::provider::MODEL)}else{None},"credit_model":"host_simulated_ceil_bytes_per_32_min_1","max_decisions":pilot::MAX_DECISIONS,"deadline_ms":scenario.deadline_ms,"goal":pilot::GOAL})
    );
    let started = Instant::now();
    let connection = qemu::Connection::start(kernel)?;
    println!(
        "{}",
        json!({"event":"guest_boot_confirmed","at_ms":started.elapsed().as_millis()})
    );
    let t = Transport {
        connection,
        request_id: 0,
        started,
        work: Work::Index,
        artifacts: crate::guest_index::History::default(),
    };
    let mut r = Run {
        t,
        scenario,
        cursor: 0,
        event_cutoff: 0,
        set: None,
        ready: BTreeMap::new(),
        spent: 0,
        decisions: 0,
        stale: 0,
        published: None,
        pending: None,
    };
    let mut outcome = r.session(policy, &mut source);
    if let Some((pending, _)) = r.pending.take() {
        // A deadline never waits to authorize late work. Drain the already-owned
        // provider afterward for usage/accounting and bounded child cleanup only.
        loop {
            match pending.poll() {
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Ok(Some(answer)) => {
                    println!(
                        "{}",
                        json!({"event":"pilot_decision_drained","call_id":r.decisions,"latency_ms":answer.latency_ms,"usage":answer.usage,"at_ms":r.now()})
                    );
                    break;
                }
                Err(error) => {
                    println!(
                        "{}",
                        json!({"event":"pilot_decision_drained","call_id":r.decisions,"error":error,"at_ms":r.now()})
                    );
                    break;
                }
            }
        }
    }
    let cleanup = r.t.snapshot(&Operation::CoordStop, "coord.stop", None);
    if !cleanup.is_ok_and(|s| s.state == "stopped") {
        outcome = Err("guest_cleanup_failed".into());
    }
    let passed = outcome.is_ok();
    let score = pilot::score(r.scenario.events.last().unwrap(), &r.ready, passed);
    println!(
        "{}",
        json!({"event":"pilot_finished","passed":passed,"reason":outcome.err(),"completion_ms":if passed{r.published.map(|(_,at)|at)}else{None},"total_elapsed_ms":r.now(),"weighted_on_time":score,"spent_credits":r.spent,"worker_jobs":r.ready.len(),"decisions":r.decisions,"stale":r.stale,"ready":r.ready,"final_snapshot":r.set})
    );
    Ok(())
}

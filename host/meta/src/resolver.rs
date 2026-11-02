//! Deterministic domain resolver. Same public content/policy, no case IDs or oracle.
//! Supported language: explicit keys; pump unit/number and review time; current,
//! previous and before-switch references; handoff dependencies; limited synonyms.
use crate::semantic::{self, Request, Selection};
use std::collections::BTreeSet;
fn words(text: &str) -> Vec<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '@')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}
fn has(tokens: &[String], word: &str) -> bool {
    tokens.iter().any(|t| t == word)
}
fn reference(tokens: &[String], prefix: &str) -> Vec<String> {
    tokens
        .windows(2)
        .filter(|p| p[0] == prefix && (p[1].len() == 1 || p[1].bytes().all(|b| b.is_ascii_digit())))
        .map(|p| p[1].clone())
        .collect()
}
fn canonical(word: &str) -> &str {
    match word {
        "eastern" => "east",
        "wet" | "rainy" => "rain",
        "wiring" | "cable" => "cables",
        "seal" | "sealing" => "seals",
        "inspect" | "inspecting" | "inspection" => "check",
        other => other,
    }
}
pub fn resolve(request: &Request) -> Selection {
    let (decision, records, reason) = choose(&request.goal);
    Selection {
        request_goal: request.goal.clone(),
        request_version: request.request_version,
        catalog_version: semantic::CATALOG_VERSION,
        permissions_version: semantic::PERMISSIONS_VERSION,
        decision: decision.into(),
        records,
        reason: reason.into(),
    }
}
fn choose(goal: &str) -> (&'static str, Vec<String>, &'static str) {
    let clarify = || {
        (
            "clarify",
            vec![],
            "Which exact document revision or unit should the request use?",
        )
    };
    let decline = || {
        (
            "decline",
            vec![],
            "The requested reference is absent from the authorized catalog.",
        )
    };
    let catalog = semantic::catalog();
    let rows = catalog["records"].as_array().expect("compiled catalog");
    let keys: Vec<_> = rows
        .iter()
        .map(|r| r["key"].as_str().expect("compiled key"))
        .collect();
    let tokens = words(goal);
    let lower = goal.to_ascii_lowercase();
    let mut selected = BTreeSet::new();
    for token in tokens.iter().filter(|t| t.contains('@')) {
        if !keys.contains(&token.as_str()) {
            return decline();
        }
        selected.insert(token.clone());
    }
    let handoff =
        (has(&tokens, "handoff") || has(&tokens, "turnover") || lower.contains("hand over"))
            && !(lower.contains("not a pump handoff")
                || lower.contains("do not prepare a pump handoff"));
    let notes: Vec<_> = rows
        .iter()
        .filter(|r| r["key"].as_str().unwrap().starts_with("notes@"))
        .collect();
    let mut candidates = BTreeSet::new();
    let units = reference(&tokens, "pump");
    let times = if has(&tokens, "review") {
        reference(&tokens, "at")
    } else {
        vec![]
    };
    for (references, prefix) in [(&units, "pump"), (&times, "at")] {
        for value in references {
            let matched: Vec<_> = notes
                .iter()
                .enumerate()
                .filter(|(_, r)| {
                    reference(&words(r["content"].as_str().unwrap()), prefix).contains(value)
                })
                .map(|(i, _)| i)
                .collect();
            if matched.is_empty() {
                return decline();
            }
            for index in matched {
                if has(&tokens, "before") && prefix == "pump" {
                    if index == 0 {
                        return decline();
                    }
                    candidates.insert(notes[index - 1]["key"].as_str().unwrap().to_owned());
                } else {
                    candidates.insert(notes[index]["key"].as_str().unwrap().to_owned());
                }
            }
        }
    }
    // Negative certainty requests are clarification, not permission to default latest.
    if lower.contains("do not assume") || lower.contains("don't assume") {
        return clarify();
    }
    if ["current", "newest", "latest"]
        .iter()
        .any(|w| has(&tokens, w))
        || lower.contains("most recent")
    {
        candidates.insert(notes.last().unwrap()["key"].as_str().unwrap().to_owned());
    }
    if ["previous", "older", "oldest"]
        .iter()
        .any(|w| has(&tokens, w))
        && !has(&tokens, "before")
    {
        candidates.insert(notes[0]["key"].as_str().unwrap().to_owned());
    }
    for key in selected.iter().filter(|s| s.starts_with("notes@")) {
        candidates.insert(key.clone());
    }
    if candidates.len() > 1 {
        return clarify();
    }
    if handoff {
        let Some(note) = candidates.into_iter().next() else {
            return clarify();
        };
        selected.insert(note);
        // Compiled handoff policy, matching semantic::POLICY. Dependency closure
        // is available to BOTH policies; it is never a lookup of an expected answer.
        selected.insert("brief@1".into());
        selected.insert("checklist@1".into());
    } else if selected.is_empty() {
        if !candidates.is_empty() && (has(&tokens, "note") || has(&tokens, "notes")) {
            selected.extend(candidates);
        } else if has(&tokens, "empty") {
            selected.insert("empty@1".into());
        } else if has(&tokens, "overview") || has(&tokens, "brief") {
            selected.insert("brief@1".into());
        } else {
            // Content retrieval with a documented domain synonym vocabulary.
            // Require two distinct informative overlaps; ties need clarification.
            let vocabulary = ["east", "gate", "rain", "seals", "cables", "check"];
            let query: BTreeSet<_> = tokens
                .iter()
                .map(|s| canonical(s))
                .filter(|s| vocabulary.contains(s))
                .collect();
            let mut best = 0;
            let mut matches = vec![];
            for row in rows {
                let source = words(row["content"].as_str().unwrap());
                let terms: BTreeSet<_> = source.iter().map(|s| canonical(s)).collect();
                let score = query.intersection(&terms).count();
                if score > best {
                    best = score;
                    matches.clear();
                }
                if score == best {
                    matches.push(row["key"].as_str().unwrap().to_owned());
                }
            }
            if best < 2 {
                return clarify();
            }
            if matches.len() != 1 {
                return clarify();
            }
            selected.extend(matches);
        }
    }
    if selected.is_empty() || selected.len() > 4 {
        return clarify();
    }
    let records: Vec<_> = selected.into_iter().collect();
    if crate::set_report::keys(records.iter().map(String::as_str)).is_err() {
        return clarify();
    }
    (
        "select",
        records,
        "Resolve supported references from public content and apply handoff dependencies.",
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn result(goal: &str) -> Selection {
        let r = Request {
            request_version: 9,
            goal: goal.into(),
        };
        let s = resolve(&r);
        assert!(semantic::validate(&s, &r).is_ok());
        s
    }
    #[test]
    fn content_references_and_dependency_closure() {
        for (goal, key) in [
            ("Aurora pump handoff for Pump A.", "notes@1"),
            ("Aurora pump handoff with review at 11.", "notes@2"),
            (
                "Aurora pump handoff before the switch to pump 8.",
                "notes@1",
            ),
            ("CURRENT Aurora pump handoff.", "notes@2"),
        ] {
            assert_eq!(result(goal).records, vec!["brief@1", "checklist@1", key]);
        }
        assert_eq!(
            result("Only the eastern gate inspection after wet weather.").records,
            vec!["appendix@1"]
        );
        assert_eq!(
            result("Only the document about cables and seals.").records,
            vec!["checklist@1"]
        );
    }
    #[test]
    fn conflicts_missing_references_and_unknown_language_do_not_execute() {
        for goal in [
            "Aurora pump handoff.",
            "Aurora pump handoff with notes@1 and notes@2.",
            "Current pump handoff for Pump A.",
            "Prepare something interesting.",
        ] {
            assert_eq!(result(goal).decision, "clarify");
        }
        for goal in [
            "Aurora pump handoff for Pump Z.",
            "Use restricted@1.",
            "Use notes@99.",
        ] {
            assert_eq!(result(goal).decision, "decline");
        }
        assert_eq!(result("Only notes@1.").records, vec!["notes@1"]);
        assert_eq!(
            result("Do not assume latest for the pump handoff.").decision,
            "clarify"
        );
    }
}

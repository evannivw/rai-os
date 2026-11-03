//! Bounded file planning contract. No guest writes or host filesystem mutation.
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub path: String,
    pub bytes: usize,
    pub text: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub request_id: u32,
    pub generation: u32,
    pub goal: String,
    pub files: Vec<File>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub kind: String,
    pub source: String,
    pub destination: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub request_id: u32,
    pub generation: u32,
    pub decision: String,
    pub actions: Vec<Action>,
    pub reason: String,
}
fn path(s: &str) -> bool {
    s.starts_with('/')
        && s.len() > 1
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b))
        && !s[1..]
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
}
pub fn request(text: &str) -> Result<Request, &'static str> {
    let r: Request = serde_json::from_str(text).map_err(|_| "invalid_file_request")?;
    if r.request_id == 0
        || r.generation == 0
        || r.goal.trim().is_empty()
        || r.goal.len() > 128
        || !r.goal.bytes().all(|b| (32..=126).contains(&b))
        || r.files.len() > 8
    {
        return Err("invalid_file_request");
    }
    for (i, f) in r.files.iter().enumerate() {
        if !path(&f.path)
            || f.bytes > 512
            || f.text.as_ref().is_some_and(|t| t.len() != f.bytes)
            || r.files[..i].iter().any(|p| p.path == f.path)
        {
            return Err("invalid_file_catalog");
        }
    }
    Ok(r)
}
pub fn parse(text: &str) -> Result<Plan, &'static str> {
    serde_json::from_str(text).map_err(|_| "invalid_file_plan")
}
pub fn validate(p: &Plan, r: &Request) -> Result<(), &'static str> {
    if p.request_id != r.request_id || p.generation != r.generation {
        return Err("stale_file_plan");
    }
    if p.reason.is_empty()
        || p.reason.len() > 160
        || !p.reason.bytes().all(|b| (32..=126).contains(&b))
    {
        return Err("invalid_file_reason");
    }
    if p.decision == "clarify" || p.decision == "decline" {
        return if p.actions.is_empty() {
            Ok(())
        } else {
            Err("nonexecuting_plan_has_actions")
        };
    }
    if p.decision != "plan" || p.actions.is_empty() || p.actions.len() > 3 {
        return Err("invalid_file_actions");
    }
    let mut copies = 0;
    for (i, a) in p.actions.iter().enumerate() {
        if !matches!(a.kind.as_str(), "copy" | "move")
            || !path(&a.source)
            || !path(&a.destination)
            || !r.files.iter().any(|f| f.path == a.source)
            || r.files.iter().any(|f| f.path == a.destination)
            || p.actions[..i]
                .iter()
                .any(|b| b.source == a.source || b.destination == a.destination)
        {
            return Err("invalid_file_action");
        }
        copies += usize::from(a.kind == "copy");
    }
    if copies + r.files.len() > 8 {
        return Err("file_capacity");
    }
    Ok(())
}
pub fn run(request_path: &std::path::Path, replay: Option<&std::path::Path>) -> Result<(), String> {
    let r = request(&crate::config::bounded_file(request_path, 32768)?)?;
    let p = if let Some(path) = replay {
        parse(&crate::config::bounded_file(path, 4096)?)?
    } else {
        let key = crate::config::load()?;
        crate::decision_source::report(crate::provider::files(&key, &r), u64::from(r.request_id))?
            .proposal
    };
    validate(&p, &r)?;
    println!(
        "{}",
        serde_json::json!({"event":"file_plan","source":if replay.is_some(){"replay"}else{"meta"},"plan":p,"execution":false})
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_colliding_and_nonexecuting_actions_are_rejected() {
        let r=request(r#"{"request_id":2,"generation":3,"goal":"Keep originals","files":[{"path":"/a","bytes":1,"text":"A"}]}"#).unwrap();
        let mut p=parse(r#"{"request_id":2,"generation":3,"decision":"plan","actions":[{"kind":"copy","source":"/a","destination":"/b"}],"reason":"Keep originals."}"#).unwrap();
        validate(&p, &r).unwrap();
        p.generation = 2;
        assert!(validate(&p, &r).is_err());
        p.generation = 3;
        p.actions[0].destination = "/a".into();
        assert!(validate(&p, &r).is_err());
        p.actions[0].destination = "/b".into();
        p.decision = "clarify".into();
        assert!(validate(&p, &r).is_err());
        p.actions.clear();
        validate(&p, &r).unwrap();
    }
}

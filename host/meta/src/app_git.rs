//! Scoped Git requests for installed app definitions, never arbitrary commands.
use crate::{apps, git_store::Store};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub operation: String,
    pub revision: String,
    pub message: String,
    pub reason: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pending {
    pub request_id: u32,
    pub base_revision: u32,
    pub app_id: String,
    pub head: String,
    pub choice: Choice,
}
pub fn parse(text: &str) -> Result<Choice, &'static str> {
    serde_json::from_str(text).map_err(|_| "invalid_app_git_choice")
}
fn text(value: &str, max: usize, nonempty: bool) -> bool {
    value.len() <= max
        && (!nonempty || !value.trim().is_empty())
        && value.bytes().all(|b| (32..=126).contains(&b))
}
pub fn validate(choice: &Choice, context: &crate::git_store::Context) -> Result<(), &'static str> {
    if !text(&choice.reason, 120, true) || !text(&choice.message, 80, false) {
        return Err("invalid_app_git_text");
    }
    match choice.operation.as_str() {
        "status" | "history" | "clarify"
            if choice.revision.is_empty() && choice.message.is_empty() =>
        {
            Ok(())
        }
        "save" if choice.revision.is_empty() && !choice.message.trim().is_empty() => Ok(()),
        "show" | "diff" | "restore"
            if choice.message.is_empty()
                && context.history.iter().any(|v| v.oid == choice.revision) =>
        {
            Ok(())
        }
        _ => Err("app_git_operation_denied"),
    }
}
fn read_request(path: &Path) -> Result<apps::Request, String> {
    let r = apps::request(&crate::config::bounded_file(path, 16384)?)?;
    let spec = r.target.as_ref().ok_or("app_git_requires_installed_app")?;
    spec.checked()?;
    if !r.app_ids.contains(&spec.id) {
        return Err("app_git_requires_installed_app".into());
    }
    Ok(r)
}
pub fn plan(root: &Path, request_path: &Path, replay: Option<&Path>) -> Result<(), String> {
    let r = read_request(request_path)?;
    let spec = r.target.as_ref().unwrap();
    let store = Store::new(root, &spec.id)?;
    // Read-only observation before credentials are loaded. No repository creation.
    let context = store.context(spec)?;
    let choice = if let Some(path) = replay {
        parse(&crate::config::bounded_file(path, 2048)?)?
    } else {
        let key = crate::config::load()?;
        crate::decision_source::report(
            crate::provider::app_git(&key, &r, &context),
            u64::from(r.request_id),
        )?
        .proposal
    };
    validate(&choice, &context)?;
    let pending = Pending {
        request_id: r.request_id,
        base_revision: r.base_revision,
        app_id: spec.id.clone(),
        head: context.head,
        choice,
    };
    println!(
        "{}",
        json!({"event":"app_git_plan","pending":pending,"execution":false})
    );
    Ok(())
}
pub fn apply(root: &Path, request_path: &Path, pending_path: &Path) -> Result<(), String> {
    let r = read_request(request_path)?;
    let spec = r.target.as_ref().unwrap();
    let pending: Pending = serde_json::from_str(&crate::config::bounded_file(pending_path, 4096)?)
        .map_err(|_| "invalid_app_git_pending")?;
    if pending.request_id != r.request_id
        || pending.base_revision != r.base_revision
        || pending.app_id != spec.id
    {
        return Err("stale_app_git_request".into());
    }
    let store = Store::new(root, &spec.id)?;
    let _lock = store.lock()?;
    let context = store.context(spec)?;
    if pending.head != context.head {
        return Err("stale_app_git_head".into());
    }
    validate(&pending.choice, &context)?;
    let c = &pending.choice;
    let (detail, notice) = match c.operation.as_str() {
        "save" => {
            let oid = store.save(spec, &c.message, &context)?;
            (
                json!({"commit":oid,"changed":oid!=context.head}),
                format!(
                    "Git saved definition {}. Records are not versioned.",
                    &oid[..8]
                ),
            )
        }
        "status" => (
            json!(&context),
            format!(
                "Git: {} saved versions. Current definition {}.",
                context.total,
                if context.matches_current {
                    "matches HEAD"
                } else {
                    "has unsaved changes"
                }
            ),
        ),
        "history" => {
            let list = context
                .history
                .iter()
                .take(3)
                .map(|v| {
                    format!(
                        "{} {}",
                        &v.oid[..8],
                        v.message.chars().take(20).collect::<String>()
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            (
                json!(&context),
                if list.is_empty() {
                    "Git history is empty. Save this definition first.".into()
                } else {
                    format!("Git history: {list}")
                },
            )
        }
        "show" => (
            json!({"revision":c.revision,"spec":store.show(&c.revision)?}),
            format!(
                "Git definition {} exported to the session log. No app changed.",
                &c.revision[..8]
            ),
        ),
        "diff" => (
            json!({"revision":c.revision,"diff":store.diff(&c.revision, spec)?}),
            format!(
                "Git diff against {} is in the session log. No app changed.",
                &c.revision[..8]
            ),
        ),
        "restore" => {
            let old = store.show(&c.revision)?;
            let plan = apps::Plan {
                request_id: r.request_id,
                base_revision: r.base_revision,
                decision: "install".into(),
                spec: Some(old),
                reason: format!(
                    "Restore Git definition {}. Keep current records; review all changes.",
                    &c.revision[..8]
                ),
            };
            apps::validate(&plan, &r)?;
            println!(
                "{}",
                json!({"event":"app_plan","source":"git","plan":plan,"execution":false})
            );
            (
                json!({"revision":c.revision}),
                "Review the Git restore proposal. Current records are retained.".into(),
            )
        }
        "clarify" => (json!({}), c.reason.clone()),
        _ => unreachable!("validated operation"),
    };
    println!(
        "{}",
        json!({"event":"app_git_result","operation":c.operation,"detail":detail,"notice":notice,"guest_execution":false})
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_explicit_tools_and_observed_revisions_are_admitted() {
        let context = crate::git_store::Context {
            head: "a".repeat(40),
            total: 1,
            matches_current: false,
            history: vec![crate::git_store::Version {
                oid: "a".repeat(40),
                message: "v1".into(),
            }],
        };
        let mut c = Choice {
            operation: "restore".into(),
            revision: "a".repeat(40),
            message: String::new(),
            reason: "Restore previous behavior".into(),
        };
        assert!(validate(&c, &context).is_ok());
        for bad in ["HEAD", "../other", "--exec=anything", "", &"b".repeat(40)] {
            c.revision = bad.into();
            assert!(validate(&c, &context).is_err());
        }
        c.revision.clear();
        for bad in [
            "push",
            "clone",
            "reset",
            "shell",
            "config",
            "delete",
            "commit -a",
        ] {
            c.operation = bad.into();
            assert!(validate(&c, &context).is_err());
        }
        c.operation = "save".into();
        c.message = "Save reviewed program".into();
        assert!(validate(&c, &context).is_ok());
        c.message = "line\nbreak".into();
        assert!(validate(&c, &context).is_err());
        assert!(
            parse(r#"{"operation":"status","revision":"","message":"","reason":"Read","argv":[]}"#)
                .is_err()
        );
    }
}

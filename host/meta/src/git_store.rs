//! Dedicated bare Git stores. No shell, network, worktree or model-supplied paths.
use crate::apps::Spec;
use serde::Serialize;
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
const CONFIG: &str = "[core]\n\trepositoryformatversion = 0\n\tbare = true\n";
const REF: &str = "refs/heads/versions";
const LIMIT: usize = 65536;
const MAX_VERSIONS: usize = 64;
#[derive(Serialize)]
pub struct Version {
    pub oid: String,
    pub message: String,
}
#[derive(Serialize)]
pub struct Context {
    pub head: String,
    pub total: usize,
    pub matches_current: bool,
    pub history: Vec<Version>,
}
pub struct Store {
    root: PathBuf,
    path: PathBuf,
    app_id: String,
}
pub struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn oid(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn safe_path(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("app_git_invalid_root".into());
    }
    for parent in path.ancestors() {
        match fs::symlink_metadata(parent) {
            Ok(m) if m.file_type().is_symlink() || !m.is_dir() => {
                return Err("app_git_unsafe_path".into());
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("app_git_path_unavailable".into()),
        }
    }
    Ok(())
}
fn command(directory: Option<&Path>, args: &[&str], input: &[u8]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_NAME", "RAI-OS App History")
        .env("GIT_COMMITTER_NAME", "RAI-OS App History")
        .env("GIT_AUTHOR_EMAIL", "app-history@rai-os.invalid")
        .env("GIT_COMMITTER_EMAIL", "app-history@rai-os.invalid")
        .env("LC_ALL", "C")
        .args([
            "--no-pager",
            "--no-replace-objects",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "gc.auto=0",
            "-c",
            "maintenance.auto=false",
            "-c",
            "commit.gpgSign=false",
            "-c",
            "core.attributesFile=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
        ]);
    if let Some(dir) = directory {
        cmd.arg("--git-dir").arg(dir);
    }
    let mut child = cmd
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "cannot_start_git")?;
    let mut stdin = child.stdin.take().unwrap();
    let data = input.to_vec();
    let writer = thread::spawn(move || stdin.write_all(&data));
    let stdout = child.stdout.take().unwrap();
    let reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout
            .take((LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Ok(s),
            Ok(None) if started.elapsed() < Duration::from_secs(2) => {
                thread::sleep(Duration::from_millis(5))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break Err("app_git_timeout");
            }
        }
    };
    let written = writer.join().map_err(|_| "app_git_io")?;
    let bytes = reader
        .join()
        .map_err(|_| "app_git_io")?
        .map_err(|_| "app_git_io")?;
    if !status?.success() || written.is_err() {
        return Err("app_git_command_failed".into());
    }
    if bytes.len() > LIMIT {
        return Err("app_git_output_limit".into());
    }
    String::from_utf8(bytes).map_err(|_| "app_git_invalid_output".into())
}
impl Store {
    pub fn new(root: &Path, app_id: &str) -> Result<Self, String> {
        if app_id.is_empty()
            || app_id.len() > 12
            || !app_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err("app_git_invalid_id".into());
        }
        let root = if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|_| "app_git_invalid_root")?
                .join(root)
        };
        safe_path(&root)?;
        let path = root.join(format!("{app_id}.git"));
        safe_path(&path)?;
        Ok(Self {
            root,
            path,
            app_id: app_id.into(),
        })
    }
    pub fn lock(&self) -> Result<Lock, String> {
        safe_path(&self.root)?;
        fs::create_dir_all(&self.root).map_err(|_| "app_git_root_unavailable")?;
        let path = self.root.join(format!("{}.lock", self.app_id));
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| "app_git_busy")?;
        Ok(Lock(path))
    }
    fn marker(&self) -> String {
        format!("RAI_APP_GIT_V1\n{}\n", self.app_id)
    }
    fn inspect(&self) -> Result<bool, String> {
        safe_path(&self.path)?;
        if !self.path.exists() {
            return Ok(false);
        }
        // This adapter only adopts its own small, ordinary bare stores. Local
        // users with filesystem access remain trusted; this is not OS isolation.
        let mut pending = vec![self.path.clone()];
        let mut count = 0;
        while let Some(dir) = pending.pop() {
            for entry in fs::read_dir(dir).map_err(|_| "app_git_invalid_store")? {
                count += 1;
                if count > 2048 {
                    return Err("app_git_store_limit".into());
                }
                let entry = entry.map_err(|_| "app_git_invalid_store")?;
                let kind = entry.file_type().map_err(|_| "app_git_invalid_store")?;
                if kind.is_symlink() || !(kind.is_dir() || kind.is_file()) {
                    return Err("app_git_unsafe_path".into());
                }
                if kind.is_dir() {
                    pending.push(entry.path());
                }
            }
        }
        for denied in [
            "commondir",
            "objects/info/alternates",
            "objects/info/http-alternates",
            "info/grafts",
            "shallow",
            "config.worktree",
        ] {
            if self.path.join(denied).exists() {
                return Err("app_git_external_store_denied".into());
            }
        }
        if crate::config::bounded_file(&self.path.join("rai-owner"), 128)? != self.marker()
            || crate::config::bounded_file(&self.path.join("config"), 1024)? != CONFIG
            || crate::config::bounded_file(&self.path.join("HEAD"), 128)? != format!("ref: {REF}\n")
        {
            return Err("app_git_invalid_store".into());
        }
        Ok(true)
    }
    fn run(&self, args: &[&str], input: &[u8]) -> Result<String, String> {
        command(Some(&self.path), args, input)
    }
    fn bytes(spec: &Spec) -> Result<Vec<u8>, String> {
        spec.checked()?;
        let mut data = serde_json::to_vec_pretty(spec).map_err(|_| "app_git_invalid_schema")?;
        data.push(b'\n');
        if data.len() > 16384 {
            return Err("app_git_schema_limit".into());
        }
        Ok(data)
    }
    pub fn context(&self, spec: &Spec) -> Result<Context, String> {
        let empty = Context {
            head: String::new(),
            total: 0,
            matches_current: false,
            history: vec![],
        };
        if spec.id != self.app_id {
            return Err("app_git_wrong_app".into());
        }
        Self::bytes(spec)?;
        if !self.inspect()? {
            return Ok(empty);
        }
        // New stores may have no ref yet. Other Git errors must not masquerade as empty history.
        let refs = self.run(&["for-each-ref", "--format=%(objectname)", REF], b"")?;
        if refs.is_empty() {
            return Ok(empty);
        }
        let head = refs.trim().to_string();
        if !oid(&head) {
            return Err("app_git_invalid_head".into());
        }
        let lines = self.run(
            &["log", "--max-count=65", "--format=%H%x09%s", &head, "--"],
            b"",
        )?;
        let mut history = Vec::new();
        let mut total = 0;
        for line in lines.lines() {
            let (id, message) = line.split_once('\t').ok_or("app_git_invalid_history")?;
            if !oid(id) || message.len() > 80 || !message.bytes().all(|b| (32..=126).contains(&b)) {
                return Err("app_git_invalid_history".into());
            }
            total += 1;
            if total <= 16 {
                history.push(Version {
                    oid: id.into(),
                    message: message.into(),
                });
            }
        }
        if total > MAX_VERSIONS {
            return Err("app_git_version_limit".into());
        }
        let saved = self.show(&head)?;
        let matches_current = Self::bytes(&saved)? == Self::bytes(spec)?;
        Ok(Context {
            head,
            total,
            matches_current,
            history,
        })
    }
    pub fn show(&self, revision: &str) -> Result<Spec, String> {
        if !oid(revision) {
            return Err("app_git_invalid_revision".into());
        }
        let data = self.run(&["show", &format!("{revision}:app.json")], b"")?;
        if data.len() > 16384 {
            return Err("app_git_schema_limit".into());
        }
        let spec: Spec = serde_json::from_str(&data).map_err(|_| "app_git_invalid_schema")?;
        spec.checked()?;
        if spec.id != self.app_id {
            return Err("app_git_wrong_app".into());
        }
        Ok(spec)
    }
    pub fn diff(&self, revision: &str, spec: &Spec) -> Result<String, String> {
        self.show(revision)?;
        let old = self.run(
            &["rev-parse", "--verify", &format!("{revision}:app.json")],
            b"",
        )?;
        let new = self.run(&["hash-object", "-w", "--stdin"], &Self::bytes(spec)?)?;
        if !oid(old.trim()) || !oid(new.trim()) {
            return Err("app_git_invalid_object".into());
        }
        // No checkout, attributes, filters or external diff program.
        self.run(
            &[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
                old.trim(),
                new.trim(),
                "--",
            ],
            b"",
        )
    }
    pub fn save(&self, spec: &Spec, message: &str, context: &Context) -> Result<String, String> {
        let data = Self::bytes(spec)?;
        if spec.id != self.app_id
            || message.trim().is_empty()
            || message.len() > 80
            || !message.bytes().all(|b| (32..=126).contains(&b))
        {
            return Err("app_git_invalid_save".into());
        }
        // Do not trust a stale observer's matches_current flag for different
        // bytes. Reobserve before the no-op fast path as well as before commit.
        let fresh = self.context(spec)?;
        if fresh.head != context.head {
            return Err("stale_app_git_head".into());
        }
        if fresh.matches_current {
            return Ok(fresh.head);
        }
        if fresh.total >= MAX_VERSIONS {
            return Err("app_git_version_limit".into());
        }
        if !self.inspect()? {
            fs::create_dir(&self.path).map_err(|_| "app_git_create_failed")?;
            command(
                None,
                &[
                    "init",
                    "--bare",
                    "--initial-branch=versions",
                    "--template=",
                    self.path.to_str().ok_or("app_git_invalid_root")?,
                ],
                b"",
            )?;
            fs::write(self.path.join("config"), CONFIG).map_err(|_| "app_git_create_failed")?;
            fs::write(self.path.join("rai-owner"), self.marker())
                .map_err(|_| "app_git_create_failed")?;
        }
        let blob = self.run(&["hash-object", "-w", "--stdin"], &data)?;
        if !oid(blob.trim()) {
            return Err("app_git_invalid_object".into());
        }
        let tree = self.run(
            &["mktree"],
            format!("100644 blob {}\tapp.json\n", blob.trim()).as_bytes(),
        )?;
        if !oid(tree.trim()) {
            return Err("app_git_invalid_object".into());
        }
        let mut args = vec!["commit-tree", tree.trim()];
        if !context.head.is_empty() {
            args.extend(["-p", &context.head]);
        }
        args.extend(["-F", "-"]);
        let commit = self.run(&args, format!("{message}\n").as_bytes())?;
        if !oid(commit.trim()) {
            return Err("app_git_invalid_object".into());
        }
        // Compare-and-swap prevents overwriting a concurrent/newer history head.
        self.run(
            &[
                "update-ref",
                "--no-deref",
                REF,
                commit.trim(),
                if context.head.is_empty() {
                    "0000000000000000000000000000000000000000"
                } else {
                    &context.head
                },
            ],
            b"",
        )?;
        Ok(commit.trim().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "rai-git-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn spec() -> Spec {
        serde_json::from_str(r#"{"id":"counter","title":"Counter","capability":"records.local","fields":[{"key":"amount","label":"Amount","kind":"number","required":true,"default":"1"}],"filter_key":"","filter_label":""}"#).unwrap()
    }
    #[test]
    fn composed_pages_survive_real_git_roundtrip() {
        let tmp = Temp::new();
        let store = Store::new(&tmp.0, "counter").unwrap();
        let mut a = spec();
        a.pages=serde_json::from_value(serde_json::json!([{"title":"Totals","blocks":[{"kind":"metric","title":"Sum","value":"amount","label_field":"","aggregate":"sum","scope":"all"}]}])).unwrap();
        let _lock = store.lock().unwrap();
        let context = store.context(&a).unwrap();
        let id = store.save(&a, "Add visual report", &context).unwrap();
        assert_eq!(
            serde_json::to_value(store.show(&id).unwrap()).unwrap(),
            serde_json::to_value(&a).unwrap()
        );
        a.pages.clear();
        let id2 = store
            .save(&a, "Remove visual report", &store.context(&a).unwrap())
            .unwrap();
        assert_ne!(id, id2);
        assert_eq!(store.show(&id).unwrap().pages.len(), 1);
        assert!(store.show(&id2).unwrap().pages.is_empty());
    }
    #[test]
    fn transfer_rules_survive_real_git_versions() {
        let tmp = Temp::new();
        let store = Store::new(&tmp.0, "counter").unwrap();
        let mut a = spec();
        for (key, kind) in [("name", "text"), ("target", "text"), ("balance", "number")] {
            a.fields.push(crate::apps::Field {
                key: key.into(),
                label: key.into(),
                kind: kind.into(),
                required: true,
                default: "".into(),
                formula: "".into(),
            });
        }
        a.transfer = Some(crate::apps::Transfer {
            id: "move".into(),
            label: "Move".into(),
            balance_field: "balance".into(),
            amount_field: "amount".into(),
            target_field: "target".into(),
            key_field: "name".into(),
        });
        let _lock = store.lock().unwrap();
        let id = store
            .save(&a, "Add transfer", &store.context(&a).unwrap())
            .unwrap();
        assert_eq!(
            serde_json::to_value(store.show(&id).unwrap()).unwrap(),
            serde_json::to_value(&a).unwrap()
        );
        a.transfer = None;
        let id2 = store
            .save(&a, "Remove transfer", &store.context(&a).unwrap())
            .unwrap();
        assert!(store.show(&id).unwrap().transfer.is_some());
        assert!(store.show(&id2).unwrap().transfer.is_none());
    }
    #[test]
    fn real_git_versions_diff_and_compare_exchange_preserve_history() {
        let tmp = Temp::new();
        let store = Store::new(&tmp.0, "counter").unwrap();
        let a = spec();
        let empty = store.context(&a).unwrap();
        assert_eq!(empty.total, 0);
        assert!(!store.path.exists());
        let _lock = store.lock().unwrap();
        assert!(store.lock().is_err());
        let first = store.save(&a, "Original definition", &empty).unwrap();
        assert!(oid(&first));
        let current = store.context(&a).unwrap();
        assert!(current.matches_current);
        assert_eq!(current.total, 1);
        assert_eq!(store.save(&a, "No duplicate", &current).unwrap(), first);
        let mut b = a.clone();
        b.title = "Revised counter".into();
        let second = store
            .save(&b, "Revised definition", &store.context(&b).unwrap())
            .unwrap();
        assert_ne!(first, second);
        let latest = store.context(&b).unwrap();
        assert_eq!(latest.total, 2);
        assert_eq!(latest.history[0].oid, second);
        assert_eq!(latest.history[1].oid, first);
        assert_eq!(store.show(&first).unwrap().title, "Counter");
        let diff = store.diff(&first, &b).unwrap();
        assert!(diff.contains("-  \"title\": \"Counter\""));
        assert!(diff.contains("+  \"title\": \"Revised counter\""));
        let mut c = b.clone();
        c.title = "Stale attempt".into();
        assert!(
            store
                .save(&c, "Must not replace newer history", &current)
                .is_err()
        );
        assert_eq!(store.context(&b).unwrap().head, second);
        let tree = store
            .run(&["ls-tree", "--name-only", &second], b"")
            .unwrap();
        assert_eq!(tree, "app.json\n");
    }
    #[test]
    fn ordinary_repositories_and_modified_configuration_are_rejected() {
        let tmp = Temp::new();
        let store = Store::new(&tmp.0, "counter").unwrap();
        let a = spec();
        fs::create_dir(&store.path).unwrap();
        assert!(store.context(&a).is_err());
        fs::remove_dir(&store.path).unwrap();
        let _lock = store.lock().unwrap();
        store
            .save(&a, "Initial", &store.context(&a).unwrap())
            .unwrap();
        fs::write(
            store.path.join("config"),
            format!("{CONFIG}[include]\npath = /not/allowed\n"),
        )
        .unwrap();
        assert!(store.context(&a).is_err());
        fs::write(store.path.join("config"), CONFIG).unwrap();
        fs::create_dir_all(store.path.join("objects/info")).unwrap();
        fs::write(store.path.join("objects/info/alternates"), "/not/allowed\n").unwrap();
        assert!(store.context(&a).is_err());
        assert!(Store::new(&tmp.0, "../escape").is_err());
        assert!(Store::new(&tmp.0.join("../escape"), "counter").is_err());
    }
    #[test]
    fn version_limit_is_enforced_and_recent_history_stays_bounded() {
        let tmp = Temp::new();
        let store = Store::new(&tmp.0, "counter").unwrap();
        let mut a = spec();
        let _lock = store.lock().unwrap();
        for version in 0..MAX_VERSIONS {
            a.title = format!("Version {version}");
            store
                .save(&a, "Bounded history", &store.context(&a).unwrap())
                .unwrap();
        }
        let full = store.context(&a).unwrap();
        assert_eq!(full.total, 64);
        assert_eq!(full.history.len(), 16);
        assert_eq!(
            store.save(&a, "Identical content", &full).unwrap(),
            full.head
        );
        a.title = "Version sixty five".into();
        assert!(matches!(store.save(&a,"Over limit",&full),Err(e) if e=="app_git_version_limit"));
        assert_eq!(store.context(&a).unwrap().head, full.head);
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_are_denied_and_reference_hooks_do_not_execute() {
        use std::os::unix::{fs::PermissionsExt, fs::symlink};
        let tmp = Temp::new();
        let store = Store::new(&tmp.0, "counter").unwrap();
        let a = spec();
        let _lock = store.lock().unwrap();
        store
            .save(&a, "Initial", &store.context(&a).unwrap())
            .unwrap();
        let marker = tmp.0.join("hook-ran");
        let hooks = store.path.join("hooks");
        fs::create_dir(&hooks).unwrap();
        let hook = hooks.join("reference-transaction");
        fs::write(
            &hook,
            format!("#!/bin/sh\ntouch '{}'\nexit 1\n", marker.display()),
        )
        .unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
        let mut b = a.clone();
        b.title = "Changed".into();
        store
            .save(&b, "With disabled hook", &store.context(&b).unwrap())
            .unwrap();
        assert!(!marker.exists());
        symlink(&tmp.0, store.path.join("outside")).unwrap();
        assert!(store.context(&b).is_err());
        fs::remove_file(store.path.join("outside")).unwrap();
        symlink(&store.path, tmp.0.join("alias.git")).unwrap();
        assert!(Store::new(&tmp.0, "alias").is_err());
    }
}

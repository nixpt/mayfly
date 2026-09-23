//! MAYFLY-9 end to end through the real binary over temp git repos: state-dir
//! resolution (.jagent project, linked worktree, no-.jagent fallback, env/flag
//! override), runner merge precedence, unknown runners, and init-runners.
//! HOME is sandboxed per test so nothing touches the real ~/.local/state/mayfly.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mayfly-proj-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        ok.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&ok.stderr)
    );
}

/// A git repo with one commit; `jagent` adopts `.jagent/` (+ ignores .jagent/local/).
fn repo(root: &Path, jagent: bool) -> PathBuf {
    let r = root.join("repo");
    std::fs::create_dir_all(&r).unwrap();
    git(&r, &["init", "-q"]);
    if jagent {
        std::fs::create_dir_all(r.join(".jagent")).unwrap();
        std::fs::write(r.join(".gitignore"), ".jagent/local/\n").unwrap();
    }
    std::fs::write(r.join("README"), "x\n").unwrap();
    git(
        &r,
        &["-c", "user.email=t@t", "-c", "user.name=t", "add", "-A"],
    );
    git(
        &r,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "-m",
            "init",
        ],
    );
    r
}

const TASK: &str = r#"{"task": "Check exit code of the file src/x.rs probe",
  "done_when": {"type": "command", "run": "true", "expect_exit": 0},
  "harness": "exec", "ttl": "30s"}"#;

fn mayfly(cwd: &Path, home: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mayfly"));
    c.current_dir(cwd)
        .env("HOME", home)
        .env_remove("MAYFLY_STATE_DIR")
        .args(args);
    for (k, v) in env {
        c.env(k, v);
    }
    c.output().unwrap()
}

fn hatch(cwd: &Path, home: &Path, extra: &[&str], env: &[(&str, &str)]) -> Output {
    std::fs::write(cwd.join("task.json"), TASK).unwrap();
    let mut args = vec!["hatch", "task.json"];
    args.extend_from_slice(extra);
    let out = mayfly(cwd, home, &args, env);
    assert!(
        out.status.success(),
        "hatch: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn records(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().join("record.json").exists())
                .count()
        })
        .unwrap_or(0)
}

#[test]
fn state_goes_to_the_projects_jagent_local() {
    let t = tmp("state");
    let home = t.join("home");
    let r = repo(&t, true);
    hatch(&r, &home, &[], &[]);
    assert_eq!(records(&r.join(".jagent/local/mayfly")), 1);
    assert_eq!(records(&home.join(".local/state/mayfly")), 0);
    // list/status resolve the same way
    let list = mayfly(&r, &home, &["list"], &[]);
    assert_eq!(String::from_utf8_lossy(&list.stdout).lines().count(), 1);
}

#[test]
fn linked_worktree_uses_the_main_checkouts_state() {
    let t = tmp("wt");
    let home = t.join("home");
    let r = repo(&t, true);
    let wt = t.join("wt");
    git(
        &r,
        &[
            "worktree",
            "add",
            "-q",
            wt.to_str().unwrap(),
            "-b",
            "feature",
        ],
    );
    hatch(&wt, &home, &[], &[]);
    assert_eq!(
        records(&r.join(".jagent/local/mayfly")),
        1,
        "worktree hatch must land in the main checkout"
    );
    assert!(!wt.join(".jagent/local/mayfly").exists());
}

#[test]
fn no_jagent_falls_back_to_home_and_leaves_repo_clean() {
    let t = tmp("nojagent");
    let home = t.join("home");
    let r = repo(&t, false);
    hatch(&r, &home, &[], &[]);
    assert_eq!(records(&home.join(".local/state/mayfly")), 1);
    assert!(
        !r.join(".jagent").exists(),
        "must not create .jagent in an unadopted repo"
    );
}

#[test]
fn flag_beats_env_beats_project() {
    let t = tmp("override");
    let home = t.join("home");
    let r = repo(&t, true);
    let env_dir = t.join("env-state");
    let flag_dir = t.join("flag-state");
    hatch(
        &r,
        &home,
        &[],
        &[("MAYFLY_STATE_DIR", env_dir.to_str().unwrap())],
    );
    assert_eq!(records(&env_dir), 1);
    hatch(
        &r,
        &home,
        &["--state-dir", flag_dir.to_str().unwrap()],
        &[("MAYFLY_STATE_DIR", env_dir.to_str().unwrap())],
    );
    assert_eq!(records(&flag_dir), 1);
    assert_eq!(records(&env_dir), 1);
    assert_eq!(records(&r.join(".jagent/local/mayfly")), 0);
}

#[test]
fn warns_when_local_state_is_not_ignored() {
    let t = tmp("warn");
    let home = t.join("home");
    let r = repo(&t, true);
    std::fs::write(r.join(".gitignore"), "").unwrap();
    let out = hatch(&r, &home, &[], &[]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("not gitignored"));
}

#[test]
fn init_runners_is_idempotent_and_never_overwrites() {
    let t = tmp("init");
    let home = t.join("home");
    let r = repo(&t, true);
    std::fs::create_dir_all(r.join(".jagent/agents")).unwrap();
    std::fs::write(
        r.join(".jagent/agents/.manifest"),
        "# m\n\n[commit]\n\"README.md\"\n\n[local]\n\"*.lock\"\n",
    )
    .unwrap();
    let first = mayfly(&r, &home, &["init-runners"], &[]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let read = r.join(".jagent/agents/mayfly/read.json");
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&read).unwrap()).unwrap();
    assert_eq!(v["model"], "haiku");
    assert_eq!(v["read_only"], true);
    let manifest = std::fs::read_to_string(r.join(".jagent/agents/.manifest")).unwrap();
    let commit = manifest.split("[local]").next().unwrap();
    assert!(
        commit.contains("\"mayfly/*.json\"") && commit.contains("\"mayfly/README.md\""),
        "{manifest}"
    );
    std::fs::write(&read, "{\"harness\":\"claude\",\"model\":\"mine\"}").unwrap();
    let second = mayfly(&r, &home, &["init-runners"], &[]);
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("nothing overwritten"));
    assert!(std::fs::read_to_string(&read).unwrap().contains("mine"));
    assert_eq!(
        manifest,
        std::fs::read_to_string(r.join(".jagent/agents/.manifest")).unwrap(),
        "manifest edit must be idempotent"
    );
    let list = mayfly(&r, &home, &["runners"], &[]);
    assert!(String::from_utf8_lossy(&list.stdout).starts_with("read\tclaude\tmine"));
}

#[test]
fn runner_defaults_merge_under_the_task_and_task_wins() {
    let t = tmp("merge");
    let home = t.join("home");
    let r = repo(&t, true);
    let dir = r.join(".jagent/agents/mayfly");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("read.json"), r#"{"harness":"claude","model":"haiku","read_only":true,"ttl":"10m","budget":{"max_usd":0.25}}"#).unwrap();
    let task = r#"{"task": "List the files under the directory src/ by name",
        "done_when": {"type": "command", "run": "true"}, "model": "sonnet"}"#;
    std::fs::write(r.join("t.json"), task).unwrap();
    let out = mayfly(
        &r,
        &home,
        &["hatch", "t.json", "--runner", "read", "--dry-run"],
        &[],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let plan = String::from_utf8_lossy(&out.stdout);
    assert!(
        plan.contains("\"sonnet\"") && !plan.contains("\"haiku\""),
        "task model must win: {plan}"
    );
    assert!(
        plan.contains("--tools"),
        "runner's read_only must apply: {plan}"
    );
    assert!(
        plan.contains("--max-budget-usd"),
        "runner's budget must apply: {plan}"
    );
}

#[test]
fn unknown_runner_and_task_only_fields_are_rejected() {
    let t = tmp("reject");
    let home = t.join("home");
    let r = repo(&t, true);
    let dir = r.join(".jagent/agents/mayfly");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("read.json"), r#"{"harness":"exec"}"#).unwrap();
    std::fs::write(
        dir.join("bad.json"),
        r#"{"harness":"exec","task":"sneaky"}"#,
    )
    .unwrap();
    std::fs::write(r.join("t.json"), TASK).unwrap();
    let unknown = mayfly(&r, &home, &["validate", "t.json", "--runner", "nope"], &[]);
    assert_eq!(unknown.status.code(), Some(2));
    let err = String::from_utf8_lossy(&unknown.stderr);
    assert!(
        err.contains("known runners") && err.contains("read"),
        "{err}"
    );
    let bad = mayfly(&r, &home, &["validate", "t.json", "--runner", "bad"], &[]);
    assert_eq!(bad.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad.stderr).contains("must come from the task"));
}

#[test]
fn runner_commands_refuse_outside_an_adopted_repo() {
    let t = tmp("outside");
    let home = t.join("home");
    let r = repo(&t, false);
    let out = mayfly(&r, &home, &["init-runners"], &[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(!r.join(".jagent").exists());
}

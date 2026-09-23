//! End-to-end through the real binary with the `exec` harness (no LLM): the success
//! rule, and refusal of knobs a harness can't honour, both before anything spawns.
use std::process::Command;

fn run(state: &std::path::Path, task: serde_json::Value, sub: &str) -> std::process::Output {
    let f = state.join(format!("task-{}.json", sub.len() + task.to_string().len()));
    std::fs::write(&f, task.to_string()).unwrap();
    Command::new(env!("CARGO_BIN_EXE_mayfly"))
        .args([
            "--state-dir",
            state.to_str().unwrap(),
            sub,
            f.to_str().unwrap(),
        ])
        .output()
        .unwrap()
}

fn state(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("mayfly-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn exec_success_follows_expect_exit() {
    let s = state("exec");
    let ok = run(
        &s,
        serde_json::json!({"task": "Check exit code of the file src/x.rs probe",
            "done_when": {"type": "command", "run": "exit 3", "expect_exit": 3},
            "harness": "exec", "ttl": "30s"}),
        "hatch",
    );
    assert!(
        ok.status.success(),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
    let rep: serde_json::Value = serde_json::from_slice(&ok.stdout).unwrap();
    assert_eq!(rep["success"], true);

    let bad = run(
        &s,
        serde_json::json!({"task": "Check exit code of the file src/y.rs probe",
            "done_when": {"type": "command", "run": "exit 0", "expect_exit": 3},
            "harness": "exec", "ttl": "30s"}),
        "hatch",
    );
    assert!(!bad.status.success());
    let rep: serde_json::Value = serde_json::from_slice(&bad.stdout).unwrap();
    assert_eq!(rep["success"], false);
    let _ = std::fs::remove_dir_all(s);
}

#[test]
fn unsupported_read_only_is_refused_before_spawn() {
    let s = state("ro");
    let out = run(
        &s,
        serde_json::json!({"task": "Read the file src/lib.rs and list its pub fns",
            "done_when": {"type": "command", "run": "true"},
            "harness": "exec", "read_only": true}),
        "validate",
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no enforceable read-only mode"));
    let hatch = run(
        &s,
        serde_json::json!({"task": "Read the file src/lib.rs and list its pub fns",
            "done_when": {"type": "command", "run": "true"},
            "harness": "exec", "read_only": true}),
        "hatch",
    );
    assert!(!hatch.status.success());
    let dirs = std::fs::read_dir(&s)
        .unwrap()
        .filter(|e| e.as_ref().unwrap().path().is_dir())
        .count();
    assert_eq!(dirs, 0, "a refused task must not leave a hatch dir");
    let _ = std::fs::remove_dir_all(s);
}

//! Per-project runners and state, in the `.jagent` v2 layout (MAYFLY-9).
//!
//! - **State** (hatch records) lives in `<main checkout>/.jagent/local/mayfly/`. That
//!   directory is gitignored and local to one clone, and is shared by all its linked
//!   worktrees. A repo that hasn't adopted `.jagent/` falls back to
//!   `~/.local/state/mayfly`, so mayfly never creates noise in an unadopted checkout.
//! - **Runners** are named partial tasks of defaults (harness, model, read_only, ttl,
//!   budget, …), committed in `<main checkout>/.jagent/agents/mayfly/<name>.json`.
//!   `hatch --runner <name>` merges one under the task, and the task's fields win.
//!
//! `.jagent/state/` is reserved for squadron and never used here.

use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const RUNNERS_DIR: &str = ".jagent/agents/mayfly";
pub const STATE_DIR: &str = ".jagent/local/mayfly";

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The main checkout owning `dir`. A linked worktree resolves to its main checkout.
pub fn main_checkout(dir: &Path) -> Option<PathBuf> {
    let common = PathBuf::from(git(
        dir,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?);
    if common.file_name().is_some_and(|n| n == ".git") {
        common.parent().map(Path::to_path_buf)
    } else {
        git(dir, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
    }
}

/// The project root: the main checkout, but only if it has adopted `.jagent/`.
pub fn project_root(dir: &Path) -> Option<PathBuf> {
    main_checkout(dir).filter(|root| root.join(".jagent").is_dir())
}

/// Where a hatch's records go, plus a warning to print (if any).
///
/// Precedence: explicit (`--state-dir` or `MAYFLY_STATE_DIR`, already merged by clap)
/// > `<project>/.jagent/local/mayfly` > `$HOME/.local/state/mayfly`.
pub fn resolve_state_dir(
    explicit: Option<PathBuf>,
    cwd: &Path,
    home: Option<&Path>,
) -> Result<(PathBuf, Option<String>)> {
    if let Some(p) = explicit {
        return Ok((p, None));
    }
    if let Some(root) = project_root(cwd) {
        let dir = root.join(STATE_DIR);
        let warn = (!is_ignored(&root, STATE_DIR)).then(|| {
            format!(
                "warning: {} is not gitignored in {} — add `.jagent/local/` to its .gitignore (jagent-adopt does)",
                STATE_DIR,
                root.display()
            )
        });
        return Ok((dir, warn));
    }
    let home = home.context("HOME not set")?;
    Ok((home.join(".local/state/mayfly"), None))
}

fn is_ignored(root: &Path, rel: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["check-ignore", "-q", "--no-index", rel])
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// A named runner: its file stem and parsed defaults.
pub struct Runner {
    pub name: String,
    pub defaults: Map<String, Value>,
}

/// Fields a runner may not set: the purpose and the stop condition belong to the task.
const TASK_ONLY: &[&str] = &["task", "done_when"];

pub fn load_runner(root: &Path, name: &str) -> Result<Runner> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!("invalid runner name {name:?}");
    }
    let path = root.join(RUNNERS_DIR).join(format!("{name}.json"));
    let raw =
        fs::read_to_string(&path).with_context(|| format!("read runner {}", path.display()))?;
    let v: Value =
        serde_json::from_str(&raw).with_context(|| format!("parse runner {}", path.display()))?;
    let Value::Object(defaults) = v else {
        bail!("runner {} is not a JSON object", path.display());
    };
    for k in TASK_ONLY {
        if defaults.contains_key(*k) {
            bail!("runner {name} sets `{k}`: that field must come from the task, not the runner");
        }
    }
    Ok(Runner {
        name: name.to_string(),
        defaults,
    })
}

pub fn list_runners(root: &Path) -> Result<Vec<Runner>> {
    let dir = root.join(RUNNERS_DIR);
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    let mut names: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.strip_suffix(".json").map(str::to_string)
        })
        .collect();
    names.sort();
    for n in names {
        out.push(load_runner(root, &n)?);
    }
    Ok(out)
}

/// Deep-merge the runner's defaults under the task: task keys win; nested objects
/// (budget, constraints, aging) merge key by key; arrays and scalars are replaced.
pub fn merge_under(defaults: &Map<String, Value>, task: Value) -> Value {
    fn merge(base: &Value, over: Value) -> Value {
        match (base, over) {
            (Value::Object(b), Value::Object(o)) => {
                let mut out = b.clone();
                for (k, v) in o {
                    let merged = match out.get(&k) {
                        Some(existing) => merge(existing, v),
                        None => v,
                    };
                    out.insert(k, merged);
                }
                Value::Object(out)
            }
            (_, over) => over,
        }
    }
    merge(&Value::Object(defaults.clone()), task)
}

/// One-line summary of a runner for `mayfly runners`.
pub fn describe(r: &Runner) -> String {
    let s = |k: &str| {
        r.defaults
            .get(k)
            .and_then(Value::as_str)
            .unwrap_or("-")
            .to_string()
    };
    let ro = r
        .defaults
        .get("read_only")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    format!(
        "{}\t{}\t{}\tread_only={}\tttl={}",
        r.name,
        s("harness"),
        s("model"),
        ro,
        s("ttl")
    )
}

pub const DEFAULT_READ_RUNNER: &str = r#"{
  "harness": "claude",
  "model": "haiku",
  "read_only": true,
  "ttl": "10m",
  "budget": { "max_usd": 0.25 }
}
"#;

pub const RUNNERS_README: &str = "# mayfly runners\n\n\
Each `<name>.json` here is a **runner**: a partial mayfly task of defaults (harness, model,\n\
read_only, ttl, budget, constraints, aging). `mayfly hatch task.json --runner <name>` merges\n\
it under the task. The task's fields win, and nested objects merge key by key. `task` and\n\
`done_when` always come from the task. A runner file that sets them is rejected.\n\n\
- `read.json`: the read tier. Claude Haiku, read-only (Read/Grep/Glob only, enforced by\n\
  harness flags), 10 minute TTL, $0.25 budget.\n\n\
`mayfly runners` lists them. Hatch records for this project go to `.jagent/local/mayfly/`\n\
(gitignored). Scaffolded by `mayfly init-runners`, which never overwrites a file.\n";

/// Scaffold `.jagent/agents/mayfly/` in the project. Never overwrites. Returns what was
/// written, plus notes (e.g. gitignore / manifest coverage).
pub fn init_runners(root: &Path) -> Result<(Vec<PathBuf>, Vec<String>)> {
    let dir = root.join(RUNNERS_DIR);
    fs::create_dir_all(&dir)?;
    let mut written = Vec::new();
    for (name, body) in [
        ("read.json", DEFAULT_READ_RUNNER),
        ("README.md", RUNNERS_README),
    ] {
        let p = dir.join(name);
        if !p.exists() {
            fs::write(&p, body)?;
            written.push(p);
        }
    }
    let mut notes = Vec::new();
    if !is_ignored(root, STATE_DIR) {
        notes.push(format!(
            "{STATE_DIR} is not gitignored here — add `.jagent/local/` to .gitignore (jagent-adopt does)"
        ));
    }
    // .jagent v2: files under .jagent/agents/ are LOCAL unless listed in .manifest [commit].
    let manifest = root.join(".jagent/agents/.manifest");
    let want = ["\"mayfly/*.json\"", "\"mayfly/README.md\""];
    match fs::read_to_string(&manifest) {
        Ok(text) => {
            let missing: Vec<&str> = want.iter().copied().filter(|w| !text.contains(w)).collect();
            if !missing.is_empty() {
                let updated = if let Some(pos) = text.find("[commit]") {
                    let after = pos + "[commit]".len();
                    let (head, tail) = text.split_at(after);
                    format!("{head}\n{}{tail}", missing.join("\n"))
                } else {
                    format!("{text}\n[commit]\n{}\n", missing.join("\n"))
                };
                fs::write(&manifest, updated)?;
                notes.push(format!("added {} to .jagent/agents/.manifest [commit]", missing.join(", ")));
            }
        }
        Err(_) => notes.push(
            "no .jagent/agents/.manifest: runner files are local by default in .jagent v2 — list `mayfly/*.json` under [commit] to share them (jagent-adopt creates the manifest)".into(),
        ),
    }
    Ok((written, notes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn task_wins_and_objects_merge_key_by_key() {
        let Value::Object(d) = json!({"harness":"claude","model":"haiku","read_only":true,
            "ttl":"10m","budget":{"max_usd":0.25,"max_turns":5}})
        else {
            unreachable!()
        };
        let merged = merge_under(
            &d,
            json!({"task":"t","done_when":{"type":"command","run":"true"},
            "model":"sonnet","budget":{"max_usd":1.0}}),
        );
        assert_eq!(merged["model"], "sonnet");
        assert_eq!(merged["harness"], "claude");
        assert_eq!(merged["read_only"], true);
        assert_eq!(merged["budget"]["max_usd"], 1.0);
        assert_eq!(merged["budget"]["max_turns"], 5);
    }
}

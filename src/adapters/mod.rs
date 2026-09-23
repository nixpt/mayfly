//! Harness adapters — argv only. Policy lives in mayfly.

mod ccf;
mod cece;
mod claude;
mod codex;
mod cursor;
mod cxf;
mod exec;
mod opencode;

use crate::task::{Harness, MayflyTask};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone, Serialize)]
pub struct SpawnPlan {
    pub harness: String,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub prompt_path: PathBuf,
}

pub trait Adapter {
    fn name(&self) -> &'static str;
    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan>;
}

pub fn for_harness(h: Harness) -> Box<dyn Adapter> {
    match h {
        Harness::Claude => Box::new(claude::Claude),
        Harness::Ccf => Box::new(ccf::Ccf),
        Harness::Cursor => Box::new(cursor::Cursor),
        Harness::Codex => Box::new(codex::Codex),
        Harness::Cxf => Box::new(cxf::Cxf),
        Harness::Cece => Box::new(cece::Cece),
        Harness::Opencode => Box::new(opencode::Opencode),
        Harness::Exec => Box::new(exec::Exec),
    }
}

/// Harnesses whose CLI takes a model flag (verified against each CLI's --help, s463).
fn takes_model(h: Harness) -> bool {
    matches!(
        h,
        Harness::Claude | Harness::Ccf | Harness::Codex | Harness::Cxf | Harness::Opencode
    )
}

/// Harnesses with an enforceable read-only mode: claude `--tools` + `dontAsk`,
/// codex `--sandbox read-only`. Everything else would run writable.
fn has_read_only(h: Harness) -> bool {
    matches!(
        h,
        Harness::Claude | Harness::Ccf | Harness::Codex | Harness::Cxf
    )
}

/// Refuse a task whose knobs this harness cannot actually honour (MAYFLY-8).
/// A silently ignored `model`, `read_only` or `max_usd` is worse than a refusal:
/// the caller would believe a cheap, read-only, capped run happened when it didn't.
pub fn check_supported(task: &MayflyTask) -> Result<()> {
    let h = task.harness;
    if task.model.is_some() && !takes_model(h) {
        bail!(
            "harness '{h}' has no model flag; drop `model` or pick claude/ccf/codex/cxf/opencode"
        );
    }
    if task.read_only && !has_read_only(h) {
        bail!("harness '{h}' has no enforceable read-only mode; use claude/ccf/codex/cxf for read_only tasks");
    }
    if let Some(b) = &task.budget {
        if b.max_usd.is_some() && !matches!(h, Harness::Claude | Harness::Ccf) {
            bail!("budget.max_usd is only enforceable on claude/ccf (--max-budget-usd); harness '{h}' would ignore it");
        }
    }
    Ok(())
}

/// Notes about knobs that are accepted but not enforced by any harness.
pub fn unenforced(task: &MayflyTask) -> Vec<String> {
    let mut notes = vec![];
    if task.budget.as_ref().is_some_and(|b| b.max_turns.is_some()) {
        notes.push("budget.max_turns is not enforced: no harness CLI exposes a turn cap (claude has none in --help)".into());
    }
    notes
}

/// Shared argv for the claude CLI (claude + ccf harnesses).
pub(crate) fn claude_args(task: &MayflyTask, prompt: String) -> Vec<String> {
    let mut args = vec!["-p".into(), prompt, "--output-format".into(), "text".into()];
    if task.read_only {
        // Only the read tools exist in this session, anything not pre-approved is
        // denied (headless), and no MCP servers load (an MCP tool could write).
        args.extend(
            [
                "--tools",
                "Read,Grep,Glob",
                "--allowedTools",
                "Read,Grep,Glob",
                "--permission-mode",
                "dontAsk",
                "--strict-mcp-config",
            ]
            .map(String::from),
        );
    } else {
        args.push("--dangerously-skip-permissions".into());
    }
    if let Some(m) = &task.model {
        args.push("--model".into());
        args.push(m.clone());
    }
    if let Some(usd) = task.budget.as_ref().and_then(|b| b.max_usd) {
        args.push("--max-budget-usd".into());
        args.push(format!("{usd}"));
    }
    args
}

/// Shared `codex exec` tail (codex + cxf harnesses): model + sandbox.
pub(crate) fn codex_exec_args(task: &MayflyTask) -> Vec<String> {
    let mut args = vec!["exec".into()];
    if let Some(m) = &task.model {
        args.push("-m".into());
        args.push(m.clone());
    }
    args.push("--sandbox".into());
    args.push(
        if task.read_only {
            "read-only"
        } else {
            "workspace-write"
        }
        .into(),
    );
    args.push("--skip-git-repo-check".into());
    args
}

pub fn spawn(plan: &SpawnPlan) -> Result<Child> {
    if which(&plan.program).is_none() {
        bail!(
            "harness '{}' binary `{}` not found on PATH — install it or pick another harness",
            plan.harness,
            plan.program
        );
    }
    let mut cmd = Command::new(&plan.program);
    cmd.args(&plan.args)
        .current_dir(&plan.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    cmd.spawn().with_context(|| {
        format!(
            "failed to spawn harness '{}' (`{}`)",
            plan.harness, plan.program
        )
    })
}

fn which(program: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let p = dir.join(program);
            if p.is_file() {
                Some(p)
            } else {
                None
            }
        })
    })
}

pub(crate) fn read_prompt(prompt_path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(prompt_path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Budget, DoneWhen};

    fn task(h: Harness) -> MayflyTask {
        MayflyTask {
            task: "Read src/lib.rs and report the pub fns".into(),
            done_when: DoneWhen::FilesExist {
                paths: vec!["report.md".into()],
            },
            harness: h,
            cwd: ".".into(),
            ttl: "5m".into(),
            budget: None,
            aging: None,
            artifacts: vec![],
            constraints: Default::default(),
            model: None,
            read_only: false,
        }
    }

    fn has(args: &[String], s: &str) -> bool {
        args.iter().any(|a| a == s)
    }

    #[test]
    fn claude_default_keeps_skip_permissions_and_no_model() {
        let a = claude_args(&task(Harness::Claude), "p".into());
        assert_eq!(&a[..4], ["-p", "p", "--output-format", "text"]);
        assert!(has(&a, "--dangerously-skip-permissions"));
        assert!(!has(&a, "--model") && !has(&a, "--max-budget-usd"));
    }

    #[test]
    fn claude_read_only_drops_skip_permissions_and_restricts_tools() {
        let mut t = task(Harness::Ccf);
        t.read_only = true;
        let a = claude_args(&t, "p".into());
        assert!(!has(&a, "--dangerously-skip-permissions"));
        let tools = a.iter().position(|x| x == "--tools").unwrap();
        assert_eq!(a[tools + 1], "Read,Grep,Glob");
        let mode = a.iter().position(|x| x == "--permission-mode").unwrap();
        assert_eq!(a[mode + 1], "dontAsk");
        assert!(has(&a, "--strict-mcp-config"));
    }

    #[test]
    fn claude_model_and_budget_become_flags() {
        let mut t = task(Harness::Claude);
        t.model = Some("haiku".into());
        t.budget = Some(Budget {
            max_turns: None,
            max_usd: Some(0.25),
        });
        let a = claude_args(&t, "p".into());
        let m = a.iter().position(|x| x == "--model").unwrap();
        assert_eq!(a[m + 1], "haiku");
        let b = a.iter().position(|x| x == "--max-budget-usd").unwrap();
        assert_eq!(a[b + 1], "0.25");
    }

    #[test]
    fn codex_read_only_uses_read_only_sandbox_and_model() {
        let mut t = task(Harness::Codex);
        t.read_only = true;
        t.model = Some("gpt-5".into());
        assert_eq!(
            codex_exec_args(&t),
            [
                "exec",
                "-m",
                "gpt-5",
                "--sandbox",
                "read-only",
                "--skip-git-repo-check"
            ]
        );
        assert!(codex_exec_args(&task(Harness::Cxf)).contains(&"workspace-write".to_string()));
    }

    #[test]
    fn unsupported_knobs_are_refused_not_ignored() {
        let mut t = task(Harness::Cursor);
        t.model = Some("x".into());
        assert!(check_supported(&t).is_err());

        let mut t = task(Harness::Exec);
        t.read_only = true;
        assert!(check_supported(&t).is_err());

        let mut t = task(Harness::Opencode);
        t.read_only = true;
        assert!(check_supported(&t).is_err());

        let mut t = task(Harness::Codex);
        t.budget = Some(Budget {
            max_turns: None,
            max_usd: Some(1.0),
        });
        assert!(check_supported(&t).is_err());

        let mut t = task(Harness::Opencode);
        t.model = Some("opencode/big-pickle".into());
        assert!(check_supported(&t).is_ok());
    }

    #[test]
    fn max_turns_is_reported_as_unenforced() {
        let mut t = task(Harness::Claude);
        assert!(unenforced(&t).is_empty());
        t.budget = Some(Budget {
            max_turns: Some(30),
            max_usd: None,
        });
        assert_eq!(unenforced(&t).len(), 1);
    }

    #[test]
    fn opencode_task_model_beats_env() {
        let mut t = task(Harness::Opencode);
        t.model = Some("from-task".into());
        let dir = std::env::temp_dir().join(format!("mayfly-oc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("prompt.txt");
        std::fs::write(&p, "hi").unwrap();
        let plan = for_harness(Harness::Opencode).plan(&t, &p).unwrap();
        let m = plan.args.iter().position(|x| x == "-m").unwrap();
        assert_eq!(plan.args[m + 1], "from-task");
        let _ = std::fs::remove_dir_all(dir);
    }
}

use super::mcp_config::tool_base_name;
use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// GitHub Copilot CLI (`copilot -p`), MAYFLY-16. A local process like every other
/// adapter — distinct from MAYFLY-5's Copilot *cloud* agent, which has no local PID.
///
/// Auth: the CLI reads `COPILOT_GITHUB_TOKEN` / `GH_TOKEN` / `GITHUB_TOKEN` or its own
/// `copilot login`; each prompt draws on the account's Copilot premium requests.
///
/// Permission model (live-verified 2026-10-16, Copilot CLI 1.0.98, `-p` mode):
/// - a tool needing approval is denied, never prompted — the hatch can't hang on a prompt;
/// - `--deny-tool` beats every allow rule, including `--allow-all-tools`;
/// - `write(/abs/file)` allows exactly that file; a write anywhere else is denied;
/// - even with no allow rules, the CLI auto-approves its built-in read-only shell
///   commands (`ls`, `cat`, `pwd`, `date`, `git log`, `gh pr list`); `env`, `curl`,
///   `python3`, `touch`, `rm` and `git commit` stay denied.
pub struct Copilot;

impl Adapter for Copilot {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        let prompt = super::read_prompt(prompt_path)?;
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "copilot".into(),
            args: copilot_args(task, prompt)?,
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

pub(crate) fn copilot_args(task: &MayflyTask, prompt: String) -> Result<Vec<String>> {
    let mut args = vec!["-p".into(), prompt, "-s".into()];
    if task.read_only {
        // Read tools need no approval; deny every write, every shell command (closing
        // the auto-approved read-only commands too), all URLs, and the built-in GitHub MCP.
        for kind in ["write", "shell", "url"] {
            args.push("--deny-tool".into());
            args.push(kind.into());
        }
        args.push("--disable-builtin-mcps".into());
    } else if !task.tools.is_empty() {
        let rules = tool_rules(&task.tools, &task.cwd)?;
        for rule in &rules.allow {
            args.push("--allow-tool".into());
            args.push(rule.clone());
        }
        if !rules.has_shell {
            // No Bash rule: no shell at all, not even the auto-approved read-only commands.
            args.push("--deny-tool".into());
            args.push("shell".into());
        }
        for dir in &rules.add_dirs {
            args.push("--add-dir".into());
            args.push(dir.display().to_string());
        }
        args.push("--disable-builtin-mcps".into());
    } else {
        // Unattended full access, as cursor `--yolo` / claude skip-permissions.
        args.push("--allow-all".into());
    }
    if let Some(m) = &task.model {
        args.push("--model".into());
        args.push(m.clone());
    }
    Ok(args)
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct ToolRules {
    pub allow: Vec<String>,
    pub add_dirs: Vec<PathBuf>,
    pub has_shell: bool,
}

/// Translate mayfly's Claude-syntax `tools` into Copilot permission rules, refusing any
/// rule Copilot can't enforce with the same meaning (a silently broader rule is worse
/// than a refusal — MAYFLY-8).
pub(crate) fn tool_rules(tools: &[String], cwd: &str) -> Result<ToolRules> {
    let mut out = ToolRules::default();
    for spec in tools {
        let base = tool_base_name(spec);
        let arg = spec
            .split_once('(')
            .map(|(_, rest)| rest.strip_suffix(')').unwrap_or(rest));
        match (base, arg) {
            ("Read" | "Grep" | "Glob" | "LS", None) => {} // reads need no approval
            ("Read" | "Grep" | "Glob" | "LS", Some(_)) => bail!(
                "tool '{spec}': copilot can't scope reads to a path; use a bare '{base}'"
            ),
            ("Bash", None) => {
                out.has_shell = true;
                out.allow.push("shell".into());
            }
            ("Bash", Some(cmd)) => {
                let stem = cmd.strip_suffix(":*").unwrap_or(cmd);
                if stem.is_empty() || stem.contains('*') {
                    bail!("tool '{spec}': copilot matches a command or a `prefix:*`, not a glob");
                }
                out.has_shell = true;
                out.allow.push(format!("shell({cmd})"));
            }
            ("Edit", None) => out.allow.push("write".into()),
            ("Edit", Some(path)) => {
                if path.contains('*') {
                    bail!("tool '{spec}': copilot write rules take a literal file, not a glob");
                }
                // Claude rule syntax: `//abs` is absolute, `/rel` is project-root-relative.
                let file = if let Some(abs) = path.strip_prefix("//") {
                    PathBuf::from(format!("/{abs}"))
                } else if let Some(rel) = path.strip_prefix('/') {
                    absolute(cwd)?.join(rel)
                } else {
                    // A bare relative Copilot rule matches that name in *any* directory.
                    bail!("tool '{spec}': give an absolute '//path' or a project-relative '/path'");
                };
                if let Some(dir) = file.parent() {
                    if !dir.starts_with(absolute(cwd)?) && !out.add_dirs.iter().any(|d| d == dir) {
                        out.add_dirs.push(dir.to_path_buf());
                    }
                }
                out.allow.push(format!("write({})", file.display()));
            }
            _ => bail!(
                "tool '{spec}': harness 'copilot' maps only Bash, Edit, Read, Grep, Glob and LS rules"
            ),
        }
    }
    Ok(out)
}

/// Absolute and free of `.` components: Copilot matches `write(/abs)` literally, so
/// `/repo/./out.md` would never match the file the agent writes.
fn absolute(cwd: &str) -> Result<PathBuf> {
    let p = Path::new(cwd);
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()?.join(p)
    };
    Ok(joined.components().collect())
}

//! Harness adapters — argv only. Policy lives in mayfly.

mod ccf;
mod cece;
mod claude;
mod codex;
mod copilot;
mod cursor;
mod cxf;
mod exec;
mod mcp_config;
mod opencode;

pub use mcp_config::McpConfigGuard;

use crate::task::{Harness, MayflyTask};
use anyhow::{bail, Context, Result};
use mcp_config::{
    mcp_server_from_tool, tool_base_name, validate_mcp_server_names, write_mcp_config,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Delete hatch-local mcp-config if present (expire / dry-run cleanup).
pub fn cleanup_mcp_config(hatch_dir: &Path) {
    mcp_config::remove_mcp_config(hatch_dir);
}

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
        Harness::Copilot => Box::new(copilot::Copilot),
        Harness::Exec => Box::new(exec::Exec),
    }
}

/// Harnesses whose CLI takes a model flag (verified against each CLI's --help, s463).
fn takes_model(h: Harness) -> bool {
    matches!(
        h,
        Harness::Claude
            | Harness::Ccf
            | Harness::Codex
            | Harness::Cxf
            | Harness::Opencode
            | Harness::Copilot
    )
}

/// Harnesses with an enforceable read-only mode: claude `--tools` + `dontAsk`,
/// codex `--sandbox read-only`, copilot `--deny-tool write/shell/url`. Everything else
/// would run writable.
fn has_read_only(h: Harness) -> bool {
    matches!(
        h,
        Harness::Claude | Harness::Ccf | Harness::Codex | Harness::Cxf | Harness::Copilot
    )
}

/// Refuse a task whose knobs this harness cannot actually honour (MAYFLY-8 / MAYFLY-11).
/// A silently ignored `model`, `read_only`, `tools`, or `max_usd` is worse than a refusal:
/// the caller would believe a cheap, read-only, capped run happened when it didn't.
pub fn check_supported(task: &MayflyTask) -> Result<()> {
    let h = task.harness;
    if task.model.is_some() && !takes_model(h) {
        bail!(
            "harness '{h}' has no model flag; drop `model` or pick claude/ccf/codex/cxf/opencode/copilot"
        );
    }
    if task.read_only && !has_read_only(h) {
        bail!("harness '{h}' has no enforceable read-only mode; use claude/ccf/codex/cxf/copilot for read_only tasks");
    }
    if let Some(b) = &task.budget {
        if b.max_usd.is_some() && !matches!(h, Harness::Claude | Harness::Ccf) {
            bail!("budget.max_usd is only enforceable on claude/ccf (--max-budget-usd); harness '{h}' would ignore it");
        }
    }
    check_tools_mcp(task)?;
    Ok(())
}

fn supports_tools_mcp(h: Harness) -> bool {
    matches!(h, Harness::Claude | Harness::Ccf)
}

/// copilot enforces `tools` by translating them into its own permission rules
/// (MAYFLY-16); it has no equivalent of `mcp_servers`.
fn supports_tools(h: Harness) -> bool {
    supports_tools_mcp(h) || h == Harness::Copilot
}

fn check_tools_mcp(task: &MayflyTask) -> Result<()> {
    let has_tools = !task.tools.is_empty();
    let has_mcp = !task.mcp_servers.is_empty();
    if !has_tools && !has_mcp {
        return Ok(());
    }
    if has_mcp && !supports_tools_mcp(task.harness) {
        bail!(
            "mcp_servers are only enforceable on claude/ccf; harness '{}' would ignore them",
            task.harness
        );
    }
    if !supports_tools(task.harness) {
        bail!(
            "tools/mcp_servers are only enforceable on claude/ccf/copilot; harness '{}' would ignore them",
            task.harness
        );
    }
    if has_tools && task.read_only {
        bail!("tools cannot be combined with read_only (read_only already pins Read,Grep,Glob)");
    }
    if has_mcp && !has_tools {
        bail!("mcp_servers requires tools (name every allowed MCP tool explicitly)");
    }
    for tool in &task.tools {
        if tool_base_name(tool) == "Write" && tool.contains('(') {
            bail!(
                "tool '{tool}': Claude Code never matches a path-scoped Write(...) rule; \
                 use Edit(...) (Edit rules cover every file-editing tool, including Write)"
            );
        }
        if matches!(tool_base_name(tool), "Edit" | "Read") {
            let path = tool
                .split_once('(')
                .map(|(_, rest)| rest.trim_end_matches(')'))
                .unwrap_or("");
            if path.starts_with('/') && !path.starts_with("//") {
                bail!(
                    "tool '{tool}': in a Claude Code permission rule '/{}' is relative to the \
                     project root, not absolute; write '//{}' for an absolute path",
                    path.trim_start_matches('/'),
                    path.trim_start_matches('/')
                );
            }
        }
        if let Some(server) = mcp_server_from_tool(tool) {
            if !task.mcp_servers.iter().any(|s| s == server) {
                bail!("tool '{tool}' needs mcp_servers to include '{server}'");
            }
        }
    }
    validate_mcp_server_names(&task.mcp_servers)?;
    if task.harness == Harness::Copilot && has_tools {
        copilot::tool_rules(&task.tools, &task.cwd)?;
    }
    Ok(())
}

/// Notes about knobs that are accepted but not enforced by any harness.
pub fn unenforced(task: &MayflyTask) -> Vec<String> {
    let mut notes = vec![];
    if task.budget.as_ref().is_some_and(|b| b.max_turns.is_some()) {
        notes.push("budget.max_turns is not enforced: no harness CLI exposes a turn cap (claude has none in --help)".into());
    }
    if task.harness == Harness::Copilot
        && !task.read_only
        && task.tools.iter().any(|t| tool_base_name(t) == "Bash")
    {
        notes.push("copilot also auto-approves its built-in read-only shell commands (ls, cat, pwd, git log, gh pr list, …) beyond the listed Bash rules; writes, network and other commands stay denied".into());
    }
    notes
}

/// Shared argv for the claude CLI (claude + ccf harnesses).
///
/// When `mcp_servers` is set, writes hatch-local `mcp-config.json` under `hatch_dir`
/// (0600) and adds `--mcp-config`. Caller must tear that file down after the hatch.
pub(crate) fn claude_args(
    task: &MayflyTask,
    prompt: String,
    hatch_dir: &Path,
) -> Result<Vec<String>> {
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
    } else if !task.tools.is_empty() {
        // Curated allowlist (MAYFLY-11): never skip-permissions; MCP only via mcp_servers.
        let builtins = builtin_tools_csv(&task.tools);
        args.push("--tools".into());
        args.push(builtins);
        args.push("--allowedTools".into());
        args.push(task.tools.join(","));
        args.extend(["--permission-mode", "dontAsk", "--strict-mcp-config"].map(String::from));
        if !task.mcp_servers.is_empty() {
            let path = write_mcp_config(hatch_dir, &task.mcp_servers)?;
            args.push("--mcp-config".into());
            args.push(path.display().to_string());
        }
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
    Ok(args)
}

/// Built-in tool names for `--tools` (MCP names are not built-ins).
///
/// A path-scoped `Edit(...)` rule also exposes `Write`, so the agent can create the
/// file; the `Edit(...)` permission rule still limits which paths either tool may touch.
fn builtin_tools_csv(tools: &[String]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut push = |name: &str| {
        if !out.iter().any(|s| s == name) {
            out.push(name.to_string());
        }
    };
    for spec in tools {
        let base = tool_base_name(spec);
        if base.starts_with("mcp__") {
            continue;
        }
        push(base);
        if base == "Edit" && spec.contains('(') {
            push("Write");
        }
    }
    out.join(",")
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
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
            tools: vec![],
            mcp_servers: vec![],
        }
    }

    fn has(args: &[String], s: &str) -> bool {
        args.iter().any(|a| a == s)
    }

    fn after<'a>(args: &'a [String], flag: &str) -> &'a str {
        let i = args.iter().position(|x| x == flag).unwrap();
        &args[i + 1]
    }

    fn scratch_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mayfly-args-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn with_claude_config(body: &str, f: impl FnOnce()) {
        let _g = ENV_LOCK.lock().unwrap();
        let dir = scratch_dir();
        let cfg = dir.join("claude.json");
        std::fs::write(&cfg, body).unwrap();
        let prev = std::env::var_os("MAYFLY_CLAUDE_CONFIG");
        std::env::set_var("MAYFLY_CLAUDE_CONFIG", &cfg);
        f();
        match prev {
            Some(v) => std::env::set_var("MAYFLY_CLAUDE_CONFIG", v),
            None => std::env::remove_var("MAYFLY_CLAUDE_CONFIG"),
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn claude_default_keeps_skip_permissions_and_no_model() {
        let dir = scratch_dir();
        let a = claude_args(&task(Harness::Claude), "p".into(), &dir).unwrap();
        assert_eq!(&a[..4], ["-p", "p", "--output-format", "text"]);
        assert!(has(&a, "--dangerously-skip-permissions"));
        assert!(!has(&a, "--model") && !has(&a, "--max-budget-usd"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn claude_read_only_drops_skip_permissions_and_restricts_tools() {
        let mut t = task(Harness::Ccf);
        t.read_only = true;
        let dir = scratch_dir();
        let a = claude_args(&t, "p".into(), &dir).unwrap();
        assert!(!has(&a, "--dangerously-skip-permissions"));
        assert_eq!(after(&a, "--tools"), "Read,Grep,Glob");
        assert_eq!(after(&a, "--permission-mode"), "dontAsk");
        assert!(has(&a, "--strict-mcp-config"));
        assert!(!has(&a, "--mcp-config"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn claude_tools_only_allowlist_no_skip_permissions() {
        let mut t = task(Harness::Claude);
        t.tools = vec![
            "Bash(gh pr list:*)".into(),
            "Edit(//build/tmp/mayfly/pr-survey.md)".into(),
        ];
        let dir = scratch_dir();
        let a = claude_args(&t, "p".into(), &dir).unwrap();
        assert!(!has(&a, "--dangerously-skip-permissions"));
        assert_eq!(after(&a, "--tools"), "Bash,Edit,Write");
        assert_eq!(
            after(&a, "--allowedTools"),
            "Bash(gh pr list:*),Edit(//build/tmp/mayfly/pr-survey.md)"
        );
        assert_eq!(after(&a, "--permission-mode"), "dontAsk");
        assert!(has(&a, "--strict-mcp-config"));
        assert!(!has(&a, "--mcp-config"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn claude_tools_plus_mcp_writes_config_and_flags() {
        with_claude_config(
            r#"{"mcpServers":{"mailgate":{"type":"stdio","command":"mg","env":{"K":"v"}}}}"#,
            || {
                let mut t = task(Harness::Ccf);
                t.tools = vec![
                    "mcp__mailgate__mail_read".into(),
                    "Edit(//build/tmp/mayfly/mailer-digest.md)".into(),
                ];
                t.mcp_servers = vec!["mailgate".into()];
                let dir = scratch_dir();
                let a = claude_args(&t, "p".into(), &dir).unwrap();
                assert!(!has(&a, "--dangerously-skip-permissions"));
                assert_eq!(after(&a, "--tools"), "Edit,Write");
                assert!(has(&a, "--strict-mcp-config"));
                let cfg = PathBuf::from(after(&a, "--mcp-config"));
                assert_eq!(cfg, dir.join("mcp-config.json"));
                let v: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
                assert!(v["mcpServers"]
                    .as_object()
                    .unwrap()
                    .contains_key("mailgate"));
                assert_eq!(v["mcpServers"].as_object().unwrap().len(), 1);
                cleanup_mcp_config(&dir);
                assert!(!cfg.exists());
                let _ = std::fs::remove_dir_all(dir);
            },
        );
    }

    #[test]
    fn claude_model_and_budget_become_flags() {
        let mut t = task(Harness::Claude);
        t.model = Some("haiku".into());
        t.budget = Some(Budget {
            max_turns: None,
            max_usd: Some(0.25),
        });
        let dir = scratch_dir();
        let a = claude_args(&t, "p".into(), &dir).unwrap();
        assert_eq!(after(&a, "--model"), "haiku");
        assert_eq!(after(&a, "--max-budget-usd"), "0.25");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn tools_mcp_validate_rejects_bad_combinations() {
        with_claude_config(
            r#"{"mcpServers":{"mailgate":{"type":"stdio","command":"mg"}}}"#,
            || {
                let mut t = task(Harness::Claude);
                t.tools = vec!["Read".into()];
                t.read_only = true;
                assert!(check_supported(&t)
                    .unwrap_err()
                    .to_string()
                    .contains("tools cannot be combined with read_only"));

                let mut t = task(Harness::Claude);
                t.mcp_servers = vec!["mailgate".into()];
                assert!(check_supported(&t)
                    .unwrap_err()
                    .to_string()
                    .contains("mcp_servers requires tools"));

                let mut t = task(Harness::Claude);
                t.tools = vec!["mcp__mailgate__mail_read".into()];
                assert!(check_supported(&t)
                    .unwrap_err()
                    .to_string()
                    .contains("needs mcp_servers to include 'mailgate'"));

                let mut t = task(Harness::Claude);
                t.tools = vec!["Write(/build/tmp/mayfly/x.md)".into()];
                assert!(check_supported(&t)
                    .unwrap_err()
                    .to_string()
                    .contains("use Edit(...)"));

                let mut t = task(Harness::Claude);
                t.tools = vec!["Edit(/build/tmp/mayfly/x.md)".into()];
                assert!(check_supported(&t)
                    .unwrap_err()
                    .to_string()
                    .contains("'//build/tmp/mayfly/x.md'"));

                let mut t = task(Harness::Cursor);
                t.tools = vec!["Read".into()];
                assert!(check_supported(&t)
                    .unwrap_err()
                    .to_string()
                    .contains("only enforceable on claude/ccf"));

                let mut t = task(Harness::Claude);
                t.tools = vec!["mcp__mailgate__mail_read".into()];
                t.mcp_servers = vec!["mailgate".into()];
                assert!(check_supported(&t).is_ok());
            },
        );
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
    fn copilot_modes_become_permission_flags() {
        let p = |t: &MayflyTask| copilot::copilot_args(t, "p".into()).unwrap();
        // Unrestricted: full access, like cursor --yolo.
        let a = p(&task(Harness::Copilot));
        assert_eq!(a[..3], ["-p", "p", "-s"]);
        assert!(a.contains(&"--allow-all".to_string()));

        // read_only: deny write/shell/url, no built-in MCP, never --allow-all.
        let mut t = task(Harness::Copilot);
        t.read_only = true;
        t.model = Some("gpt-5.4".into());
        let a = p(&t);
        for kind in ["write", "shell", "url"] {
            assert!(
                a.windows(2).any(|w| w[0] == "--deny-tool" && w[1] == kind),
                "{kind}"
            );
        }
        assert!(a.contains(&"--disable-builtin-mcps".to_string()));
        assert!(!a.iter().any(|x| x.starts_with("--allow")));
        assert_eq!(after(&a, "--model"), "gpt-5.4");
        assert!(check_supported(&t).is_ok());
    }

    #[test]
    fn copilot_tools_translate_or_refuse() {
        let cwd = std::env::current_dir().unwrap();
        let r = copilot::tool_rules(
            &[
                "Bash(gh pr view:*)".into(),
                "Read".into(),
                "Edit(//build/tmp/mayfly/r.md)".into(),
                "Edit(/notes/out.md)".into(),
            ],
            ".",
        )
        .unwrap();
        assert_eq!(
            r.allow,
            [
                "shell(gh pr view:*)".to_string(),
                "write(/build/tmp/mayfly/r.md)".into(),
                format!("write({})", cwd.join("notes/out.md").display()),
            ]
        );
        assert!(r.has_shell);
        // Only the out-of-project directory needs --add-dir.
        assert_eq!(r.add_dirs, [std::path::PathBuf::from("/build/tmp/mayfly")]);

        // No Bash rule: the shell is denied outright, closing the auto-approved reads.
        let mut t = task(Harness::Copilot);
        t.tools = vec!["Read".into(), "Edit(//tmp/x.md)".into()];
        let a = copilot::copilot_args(&t, "p".into()).unwrap();
        assert!(a
            .windows(2)
            .any(|w| w[0] == "--deny-tool" && w[1] == "shell"));
        assert!(unenforced(&t).is_empty());
        t.tools.push("Bash(gh pr list:*)".into());
        assert!(unenforced(&t)[0].contains("auto-approves"));

        for bad in [
            "Read(src/**)",
            "Bash(gh *)",
            "Edit(src/**)",
            "Edit(out.md)",
            "WebFetch",
            "mcp__mailgate__mail_read",
        ] {
            let mut t = task(Harness::Copilot);
            t.tools = vec![bad.into()];
            assert!(check_supported(&t).is_err(), "{bad} should be refused");
        }
        let mut t = task(Harness::Copilot);
        t.tools = vec!["Read".into()];
        t.mcp_servers = vec!["mailgate".into()];
        assert!(check_supported(&t)
            .unwrap_err()
            .to_string()
            .contains("mcp_servers are only enforceable on claude/ccf"));
        let mut t = task(Harness::Copilot);
        t.budget = Some(Budget {
            max_turns: None,
            max_usd: Some(1.0),
        });
        assert!(check_supported(&t).is_err());
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

//! Harness adapters — argv only. Policy lives in mayfly.

mod ccf;
mod cece;
mod claude;
mod codex;
mod copilot;
mod cursor;
mod cxf;
mod exec;
mod opencode;

use crate::task::{Harness, MayflyTask};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
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

/// Local or remote execution handle.
#[derive(Debug)]
pub enum HarnessHandle {
    /// A local Child process (Cursor, Claude, Codex, etc.)
    Local { child: Child },
    /// A remote GitHub API task (Copilot cloud agent, GitHub Actions job, etc.)
    Remote {
        /// Task identifier (e.g., issue number, PR number, commit SHA)
        task_id: String,
        /// Where to poll status and how
        polling_strategy: RemoteStrategy,
        /// Adapter-specific metadata (repo, branch, etc.)
        metadata: serde_json::Value,
    },
}

/// Strategy for polling a remote agent's status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RemoteStrategy {
    /// Poll GitHub Copilot custom agent assignment status via `gh` API
    CopilotCloudAgent {
        /// org/repo where the agent is defined (e.g., "my-org/.github-private")
        agent_repo: String,
        /// Issue or PR number to track
        pr_or_issue_number: u64,
    },
    /// Poll GitHub Actions check-run status for CI workflow result
    GitHubActionsCheckRun {
        /// Repository "owner/repo"
        repo: String,
        /// Commit SHA to query runs against
        commit_sha: String,
    },
}

pub trait Adapter {
    fn name(&self) -> &'static str;
    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan>;
    /// Override for adapters that spawn remote tasks instead of local processes.
    /// Default: None (use traditional spawn).
    fn spawn_remote(
        &self,
        _task: &MayflyTask,
        _prompt_path: &Path,
    ) -> Result<Option<HarnessHandle>> {
        Ok(None)
    }
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
        Harness::Copilot => Box::new(copilot::Copilot),
    }
}

pub fn spawn(plan: &SpawnPlan) -> Result<HarnessHandle> {
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
    // main's process-group isolation (MAYFLY-6): put the child in its own
    // pgid so teardown can signal the whole group, not just the direct child.
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

    // MAYFLY-5's local/remote split: a spawned process is the Local arm.
    let child = cmd.spawn().with_context(|| {
        format!(
            "failed to spawn harness '{}' (`{}`)",
            plan.harness, plan.program
        )
    })?;
    Ok(HarnessHandle::Local { child })
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

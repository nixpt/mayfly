//! GitHub Copilot custom cloud-agent adapter.
//!
//! Instead of spawning a local harness binary, this adapter:
//! 1. Creates/finds an issue or PR in the target repo
//! 2. Assigns a custom cloud agent (.github/agents/<NAME>.md) to it
//! 3. Polls the issue/PR for completion (agent comments or status)
//! 4. Returns artifacts (diff, commits) when done

use super::{Adapter, HarnessHandle, RemoteStrategy, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::{bail, Context, Result};
use serde_json::json;
use std::path::Path;

pub struct Copilot;

impl Adapter for Copilot {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        // This adapter doesn't use traditional spawn; spawn_remote is the entry point.
        // Provide a no-op plan for compatibility.
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "gh".into(),
            args: vec![],
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }

    fn spawn_remote(&self, task: &MayflyTask, prompt_path: &Path) -> Result<Option<HarnessHandle>> {
        let _prompt = super::read_prompt(prompt_path)?;

        // Extract configuration from environment.
        let agent_repo = std::env::var("MAYFLY_COPILOT_AGENT_REPO")
            .context("MAYFLY_COPILOT_AGENT_REPO not set (e.g., 'my-org/.github-private')")?;
        let target_repo = std::env::var("MAYFLY_COPILOT_REPO")
            .context("MAYFLY_COPILOT_REPO not set (e.g., 'owner/target-repo')")?;

        // Step 1: Create an issue in the target repo with the task prompt as the body.
        let (issue_number, _is_pr) = create_issue_for_task(&target_repo, task)?;

        // Step 2: Assign the custom cloud agent to the issue.
        // NOTE: GitHub's API for custom agent assignment is still being defined.
        // This is a placeholder that will be updated as the API matures.
        assign_agent(&target_repo, issue_number)?;

        // Step 3: Return a remote handle for polling.
        Ok(Some(HarnessHandle::Remote {
            task_id: format!("issue-{}", issue_number),
            polling_strategy: RemoteStrategy::CopilotCloudAgent {
                agent_repo,
                pr_or_issue_number: issue_number as u64,
            },
            metadata: json!({
                "repo": target_repo,
                "is_pr": false,
            }),
        }))
    }
}

/// Create a GitHub issue in the target repo with the mayfly task prompt as the body.
fn create_issue_for_task(repo: &str, task: &MayflyTask) -> Result<(i32, bool)> {
    // Verify `gh` is available.
    if !check_gh_available() {
        bail!("GitHub CLI 'gh' not found on PATH");
    }

    let issue_title = format!("mayfly: {}", task.task.chars().take(60).collect::<String>());
    let issue_body = format!(
        "Mayfly task:\n\n{}\n\nDone when: {}",
        task.task,
        task.done_when.summary()
    );

    let output = std::process::Command::new("gh")
        .args(&["issue", "create", "-R", repo, "-t", &issue_title, "-b", &issue_body])
        .output()
        .context("failed to run 'gh issue create'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("gh issue create failed: {}", stderr);
    }

    // Parse issue number from stdout URL (e.g., "https://github.com/owner/repo/issues/123").
    let stdout = String::from_utf8_lossy(&output.stdout);
    let issue_num = stdout
        .trim()
        .split('/')
        .find_map(|s| s.parse::<i32>().ok())
        .context("failed to parse issue number from gh output")?;

    eprintln!("mayfly: created issue #{} in {}", issue_num, repo);
    Ok((issue_num, false))  // false = not a PR (for now)
}

/// Assign a custom cloud agent to the issue.
/// This is a placeholder pending GitHub's API documentation.
fn assign_agent(repo: &str, issue_num: i32) -> Result<()> {
    let agent_name = std::env::var("MAYFLY_COPILOT_AGENT_NAME")
        .unwrap_or_else(|_| "default".into());

    eprintln!(
        "mayfly: assigning agent '{}' to issue #{} in {}",
        agent_name, issue_num, repo
    );

    // TODO: Implement actual assignment once GitHub documents the custom agent assignment API.
    // For now, log a placeholder message.
    Ok(())
}

fn check_gh_available() -> bool {
    std::process::Command::new("gh")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

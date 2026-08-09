//! Throwaway git workspaces via the public `buckets` CLI.
//!
//! Shells out to `buckets worktree create|remove` — no path-dep on buckets.
//! Flame/firefly are deliberately not used here (different layer: heat/brands/fuel).

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct WorktreeSpec {
    pub repo: PathBuf,
    /// Branch name; if None, caller should fill before create (e.g. `mayfly/<id>`).
    pub branch: Option<String>,
    pub from: Option<String>,
    /// If true, leave the worktree after hatch (debug / handoff).
    pub keep: bool,
}

#[derive(Debug, Clone)]
pub struct ProvisionedWorktree {
    pub repo: PathBuf,
    pub path: PathBuf,
    pub branch: String,
    pub keep: bool,
}

/// True if `buckets` is on PATH.
pub fn buckets_available() -> bool {
    Command::new("buckets")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn create(spec: &WorktreeSpec, branch: &str) -> Result<ProvisionedWorktree> {
    if !buckets_available() {
        bail!("buckets not found on PATH — install public nixpt/buckets for --worktree");
    }
    let repo = spec
        .repo
        .canonicalize()
        .with_context(|| format!("worktree repo {}", spec.repo.display()))?;

    let mut cmd = Command::new("buckets");
    cmd.args(["worktree", "create"])
        .arg(&repo)
        .arg(branch);
    if let Some(from) = &spec.from {
        cmd.args(["--from", from]);
    }

    let output = cmd
        .output()
        .context("failed to run buckets worktree create")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("buckets worktree create failed: {stderr}");
    }

    // buckets prints progress on stderr; path is the last non-empty stdout line.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .last()
        .map(PathBuf::from)
        .with_context(|| {
            format!(
                "buckets worktree create produced no path (stdout={stdout:?}, stderr={})",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    if !path.exists() {
        bail!("buckets reported worktree path {} but it does not exist", path.display());
    }

    eprintln!("mayfly: worktree {}", path.display());
    Ok(ProvisionedWorktree {
        repo,
        path,
        branch: branch.to_string(),
        keep: spec.keep,
    })
}

pub fn remove(wt: &ProvisionedWorktree) -> Result<()> {
    if wt.keep {
        eprintln!(
            "mayfly: keeping worktree {} (branch {})",
            wt.path.display(),
            wt.branch
        );
        return Ok(());
    }
    if !buckets_available() {
        bail!("buckets not found on PATH — cannot tear down worktree");
    }

    let status = Command::new("buckets")
        .args([
            "worktree",
            "remove",
            &wt.repo.to_string_lossy(),
            &wt.path.to_string_lossy(),
            &wt.branch,
            "--force",
        ])
        .status()
        .context("failed to run buckets worktree remove")?;
    if !status.success() {
        bail!(
            "buckets worktree remove failed for {} ({})",
            wt.path.display(),
            wt.branch
        );
    }
    eprintln!("mayfly: removed worktree {}", wt.path.display());
    Ok(())
}

/// Default branch name for a hatch id.
pub fn default_branch(id: &str) -> String {
    format!("mayfly/{id}")
}

/// True when `path` looks like a primary source checkout (`.git` directory).
pub fn looks_like_source_checkout(path: &Path) -> bool {
    path.join(".git").is_dir()
}

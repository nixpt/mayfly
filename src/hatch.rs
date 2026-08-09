//! Hatch lifecycle — validate → provision → launch → watch → expire.

use crate::adapters::{self, SpawnPlan};
use crate::prompt;
use crate::store::{HatchRecord, HatchState, HatchStore};
use crate::task::{DoneWhen, MayflyTask};
use crate::ttl::parse_ttl;
use crate::worktree::{self, ProvisionedWorktree, WorktreeSpec};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::io::Read;
use std::path::PathBuf;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct HatchOpts {
    pub worktree: Option<WorktreeSpec>,
}

#[derive(Debug, Serialize)]
pub struct HatchPlanOut {
    pub id: String,
    pub harness: String,
    pub program: String,
    pub args_preview: Vec<String>,
    pub cwd: PathBuf,
    pub ttl_secs: u64,
    pub prompt_preview: String,
    pub done_when: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_branch: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HatchReport {
    pub id: String,
    pub success: bool,
    pub state: HatchState,
    pub exit_code: Option<i32>,
    pub reason: String,
    pub stdout_tail: String,
    pub stderr_tail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
}

struct Planned {
    id: String,
    dir: PathBuf,
    spawn: SpawnPlan,
    prompt: String,
    ttl: Duration,
}

pub fn plan_hatch(
    store: &HatchStore,
    task: &MayflyTask,
    opts: &HatchOpts,
) -> Result<HatchPlanOut> {
    let planned = prepare(store, task, None)?;
    let (wt_repo, wt_branch) = match &opts.worktree {
        Some(spec) => (
            Some(spec.repo.display().to_string()),
            Some(
                spec.branch
                    .clone()
                    .unwrap_or_else(|| worktree::default_branch(&planned.id)),
            ),
        ),
        None => (None, None),
    };
    Ok(HatchPlanOut {
        id: planned.id,
        harness: planned.spawn.harness.clone(),
        program: planned.spawn.program.clone(),
        args_preview: redact_args(&planned.spawn.args),
        cwd: planned.spawn.cwd.clone(),
        ttl_secs: planned.ttl.as_secs(),
        prompt_preview: planned.prompt.chars().take(400).collect(),
        done_when: task.done_when.summary(),
        worktree_repo: wt_repo,
        worktree_branch: wt_branch,
    })
}

pub fn run_hatch(
    store: &HatchStore,
    task: &MayflyTask,
    plan_out: HatchPlanOut,
    opts: &HatchOpts,
) -> Result<HatchReport> {
    let mut task = task.clone();
    let planned = prepare(store, &task, Some(&plan_out.id))?;

    let provisioned = if let Some(spec) = &opts.worktree {
        let branch = spec
            .branch
            .clone()
            .unwrap_or_else(|| worktree::default_branch(&planned.id));
        let wt = worktree::create(spec, &branch)?;
        task.cwd = wt.path.display().to_string();
        Some(wt)
    } else {
        if worktree::looks_like_source_checkout(std::path::Path::new(&task.cwd)) {
            eprintln!(
                "mayfly: warning: cwd {} looks like a shared source checkout — \
                 prefer --worktree <repo> (buckets)",
                task.cwd
            );
        }
        None
    };

    // Re-plan spawn against the (possibly new) cwd.
    let prompt_path = planned.dir.join("prompt.txt");
    std::fs::write(&prompt_path, &planned.prompt)?;
    let adapter = adapters::for_harness(task.harness);
    let spawn = adapter.plan(&task, &prompt_path)?;

    let now = Utc::now();
    let mut rec = HatchRecord {
        id: planned.id.clone(),
        state: HatchState::Alive,
        harness: spawn.harness.clone(),
        task: task.task.clone(),
        cwd: task.cwd.clone(),
        ttl: task.ttl.clone(),
        created_at: now,
        updated_at: now,
        pid: None,
        expire_reason: None,
        dir: planned.dir.clone(),
        worktree_repo: provisioned.as_ref().map(|w| w.repo.display().to_string()),
        worktree_path: provisioned.as_ref().map(|w| w.path.display().to_string()),
        worktree_branch: provisioned.as_ref().map(|w| w.branch.clone()),
        worktree_keep: provisioned.as_ref().map(|w| w.keep).unwrap_or(false),
    };
    store.save(&rec)?;

    let mut child = adapters::spawn(&spawn).with_context(|| {
        format!("spawn {} ({})", spawn.program, spawn.harness)
    })?;
    rec.pid = Some(child.id());
    store.save(&rec)?;

    let mut report = watch(store, &mut rec, &task, &mut child, planned.ttl)?;
    report.worktree_path = provisioned.as_ref().map(|w| w.path.display().to_string());

    if let Some(wt) = provisioned {
        if let Err(e) = teardown_worktree(store, &mut rec, &wt) {
            eprintln!("mayfly: worktree teardown warning: {e:#}");
        }
    }

    Ok(report)
}

pub fn expire_hatch(store: &HatchStore, id: &str, reason: &str) -> Result<()> {
    let mut rec = store
        .get(id)?
        .with_context(|| format!("no hatch {id}"))?;
    if let Some(pid) = rec.pid {
        let _ = term_pid(pid);
    }
    if let (Some(repo), Some(path), Some(branch)) = (
        rec.worktree_repo.clone(),
        rec.worktree_path.clone(),
        rec.worktree_branch.clone(),
    ) {
        let wt = ProvisionedWorktree {
            repo: PathBuf::from(repo),
            path: PathBuf::from(path),
            branch,
            keep: rec.worktree_keep,
        };
        let _ = teardown_worktree(store, &mut rec, &wt);
    }
    rec.state = HatchState::Expired;
    rec.expire_reason = Some(reason.into());
    rec.updated_at = Utc::now();
    store.save(&rec)?;
    Ok(())
}

fn teardown_worktree(
    store: &HatchStore,
    rec: &mut HatchRecord,
    wt: &ProvisionedWorktree,
) -> Result<()> {
    worktree::remove(wt)?;
    if !wt.keep {
        rec.worktree_path = None;
    }
    rec.updated_at = Utc::now();
    store.save(rec)?;
    Ok(())
}

fn prepare(
    store: &HatchStore,
    task: &MayflyTask,
    reuse_id: Option<&str>,
) -> Result<Planned> {
    let id = reuse_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("mf-{}", &Uuid::new_v4().to_string()[..8]));
    let dir = store.root().join(&id);
    std::fs::create_dir_all(&dir)?;

    let prompt_text = prompt::render(task);
    let prompt_path = dir.join("prompt.txt");
    std::fs::write(&prompt_path, &prompt_text)?;

    let adapter = adapters::for_harness(task.harness);
    let spawn = adapter.plan(task, &prompt_path)?;
    let ttl = parse_ttl(&task.ttl)?;

    Ok(Planned {
        id,
        dir,
        spawn,
        prompt: prompt_text,
        ttl,
    })
}

fn watch(
    store: &HatchStore,
    rec: &mut HatchRecord,
    task: &MayflyTask,
    child: &mut Child,
    ttl: Duration,
) -> Result<HatchReport> {
    let started = Instant::now();
    let aging = task.aging.clone().unwrap_or_default();
    let poll = Duration::from_millis(250);

    loop {
        let life = started.elapsed().as_secs_f64() / ttl.as_secs_f64().max(0.001);

        if life >= 1.0 {
            let _ = child.kill();
            let _ = child.wait();
            rec.state = HatchState::Expired;
            rec.expire_reason = Some("ttl_exceeded".into());
            rec.updated_at = Utc::now();
            store.save(rec)?;
            return Ok(HatchReport {
                id: rec.id.clone(),
                success: false,
                state: HatchState::Expired,
                exit_code: Some(124),
                reason: "ttl_exceeded".into(),
                stdout_tail: String::new(),
                stderr_tail: String::new(),
                worktree_path: rec.worktree_path.clone(),
            });
        }

        if life >= aging.narrow_at && rec.state != HatchState::Narrowing {
            rec.state = HatchState::Narrowing;
            rec.updated_at = Utc::now();
            store.save(rec)?;
        } else if life >= aging.warn_at
            && !matches!(rec.state, HatchState::Aging | HatchState::Narrowing)
        {
            rec.state = HatchState::Aging;
            rec.updated_at = Utc::now();
            store.save(rec)?;
        }

        match child.try_wait()? {
            Some(status) => {
                let (stdout_tail, stderr_tail) = drain_output(child);
                let code = status.code();
                let is_exec = matches!(task.harness, crate::task::Harness::Exec);
                let done_ok = if is_exec {
                    status.success()
                } else {
                    check_done_when(task, Duration::from_secs(5))?
                };
                let saw_marker = stdout_tail.contains("MAYFLY_DONE")
                    || stderr_tail.contains("MAYFLY_DONE");
                let success = status.success() && (done_ok || saw_marker || is_exec);

                rec.state = if success {
                    HatchState::Done
                } else {
                    HatchState::Failed
                };
                rec.expire_reason = Some(if success {
                    "purpose fulfilled".into()
                } else {
                    format!("harness exited {code:?}")
                });
                rec.pid = None;
                rec.updated_at = Utc::now();
                store.save(rec)?;

                let _ = std::fs::write(rec.dir.join("stdout.txt"), &stdout_tail);
                let _ = std::fs::write(rec.dir.join("stderr.txt"), &stderr_tail);

                return Ok(HatchReport {
                    id: rec.id.clone(),
                    success,
                    state: rec.state,
                    exit_code: code,
                    reason: rec.expire_reason.clone().unwrap_or_default(),
                    stdout_tail: tail(&stdout_tail, 2000),
                    stderr_tail: tail(&stderr_tail, 2000),
                    worktree_path: rec.worktree_path.clone(),
                });
            }
            None => {
                if !matches!(task.harness, crate::task::Harness::Exec)
                    && check_done_when(task, Duration::from_secs(5))?
                {
                    let _ = child.kill();
                    let _ = child.wait();
                    rec.state = HatchState::Done;
                    rec.expire_reason = Some("done_when satisfied".into());
                    rec.pid = None;
                    rec.updated_at = Utc::now();
                    store.save(rec)?;
                    return Ok(HatchReport {
                        id: rec.id.clone(),
                        success: true,
                        state: HatchState::Done,
                        exit_code: Some(0),
                        reason: "done_when satisfied".into(),
                        stdout_tail: String::new(),
                        stderr_tail: String::new(),
                        worktree_path: rec.worktree_path.clone(),
                    });
                }
                thread::sleep(poll);
            }
        }
    }
}

fn check_done_when(task: &MayflyTask, timeout: Duration) -> Result<bool> {
    match &task.done_when {
        DoneWhen::Command { run, expect_exit } => {
            let secs = timeout.as_secs().max(1).to_string();
            let status = std::process::Command::new("timeout")
                .args([&secs, "sh", "-c", run])
                .current_dir(&task.cwd)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()?;
            Ok(status.code() == Some(*expect_exit))
        }
        DoneWhen::FilesExist { paths } => {
            let cwd = PathBuf::from(&task.cwd);
            Ok(paths.iter().all(|p| cwd.join(p).exists()))
        }
    }
}

fn drain_output(child: &mut Child) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_string(&mut stderr);
    }
    (stdout, stderr)
}

fn tail(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s[s.len() - max..].to_string()
    }
}

fn redact_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|a| {
            if a.len() > 120 {
                format!("<{} chars>", a.len())
            } else {
                a.clone()
            }
        })
        .collect()
}

fn term_pid(pid: u32) -> Result<()> {
    let status = std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()?;
    if !status.success() {
        bail!("kill -TERM {pid} failed");
    }
    Ok(())
}

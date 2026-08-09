//! Hatch lifecycle — validate → provision → launch → watch → expire.

use crate::adapters::{self, SpawnPlan};
use crate::prompt;
use crate::store::{HatchRecord, HatchState, HatchStore};
use crate::task::{DoneWhen, MayflyTask};
use crate::ttl::parse_ttl;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::io::Read;
use std::path::PathBuf;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

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
}

struct Planned {
    id: String,
    dir: PathBuf,
    spawn: SpawnPlan,
    prompt: String,
    ttl: Duration,
}

pub fn plan_hatch(store: &HatchStore, task: &MayflyTask) -> Result<HatchPlanOut> {
    let planned = prepare(store, task, None)?;
    Ok(HatchPlanOut {
        id: planned.id,
        harness: planned.spawn.harness.clone(),
        program: planned.spawn.program.clone(),
        args_preview: redact_args(&planned.spawn.args),
        cwd: planned.spawn.cwd.clone(),
        ttl_secs: planned.ttl.as_secs(),
        prompt_preview: planned.prompt.chars().take(400).collect(),
        done_when: task.done_when.summary(),
    })
}

pub fn run_hatch(
    store: &HatchStore,
    task: &MayflyTask,
    plan_out: HatchPlanOut,
) -> Result<HatchReport> {
    let planned = prepare(store, task, Some(&plan_out.id))?;
    let now = Utc::now();
    let mut rec = HatchRecord {
        id: planned.id.clone(),
        state: HatchState::Alive,
        harness: planned.spawn.harness.clone(),
        task: task.task.clone(),
        cwd: task.cwd.clone(),
        ttl: task.ttl.clone(),
        created_at: now,
        updated_at: now,
        pid: None,
        expire_reason: None,
        dir: planned.dir.clone(),
    };
    store.save(&rec)?;

    let prompt_path = planned.dir.join("prompt.txt");
    std::fs::write(&prompt_path, &planned.prompt)?;

    let mut child = adapters::spawn(&planned.spawn).with_context(|| {
        format!(
            "spawn {} ({})",
            planned.spawn.program, planned.spawn.harness
        )
    })?;
    rec.pid = Some(child.id());
    store.save(&rec)?;

    watch(store, &mut rec, task, &mut child, planned.ttl)
}

pub fn expire_hatch(store: &HatchStore, id: &str, reason: &str) -> Result<()> {
    let mut rec = store
        .get(id)?
        .with_context(|| format!("no hatch {id}"))?;
    if let Some(pid) = rec.pid {
        let _ = term_pid(pid);
    }
    rec.state = HatchState::Expired;
    rec.expire_reason = Some(reason.into());
    rec.updated_at = Utc::now();
    store.save(&rec)?;
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
                // Exec's child *is* the done_when command — don't re-run it.
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
                });
            }
            None => {
                // Never poll done_when for Exec — that command is already the child,
                // and re-running it (e.g. `sleep 30`) would block the TTL loop.
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
            // Cap probe time so a hanging done_when can't starve the TTL loop.
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

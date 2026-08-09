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
    cmd.spawn()
        .with_context(|| format!("failed to spawn harness '{}' (`{}`)", plan.harness, plan.program))
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

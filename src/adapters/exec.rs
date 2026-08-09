use super::{Adapter, SpawnPlan};
use crate::task::{DoneWhen, MayflyTask};
use anyhow::{bail, Result};
use std::path::Path;

/// No LLM — run `done_when` (command) directly. Useful for dry sanity / CI gates.
pub struct Exec;

impl Adapter for Exec {
    fn name(&self) -> &'static str {
        "exec"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        match &task.done_when {
            DoneWhen::Command { run, .. } => Ok(SpawnPlan {
                harness: self.name().into(),
                program: "sh".into(),
                args: vec!["-c".into(), run.clone()],
                cwd: Path::new(&task.cwd).to_path_buf(),
                prompt_path: prompt_path.to_path_buf(),
            }),
            DoneWhen::FilesExist { .. } => {
                bail!("exec harness only supports done_when.type=command")
            }
        }
    }
}

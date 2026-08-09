use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::Result;
use std::path::Path;

/// Fleet-local cece-rs adapter (stub argv; refine when wiring to live binary).
pub struct Cece;

impl Adapter for Cece {
    fn name(&self) -> &'static str {
        "cece"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "cece-rs".into(),
            args: vec![
                "--prompt-file".into(),
                prompt_path.display().to_string(),
            ],
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

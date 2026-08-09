use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::Result;
use std::path::Path;

/// Fleet `cece-rs` headless dispatch (`-p` / `--afk`).
pub struct Cece;

impl Adapter for Cece {
    fn name(&self) -> &'static str {
        "cece"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        let prompt = super::read_prompt(prompt_path)?;
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "cece-rs".into(),
            args: vec![
                "-w".into(),
                task.cwd.clone(),
                "-p".into(),
                prompt,
                "--afk".into(),
                "--output-format".into(),
                "text".into(),
            ],
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

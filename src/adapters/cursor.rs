use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::Result;
use std::path::Path;

pub struct Cursor;

impl Adapter for Cursor {
    fn name(&self) -> &'static str {
        "cursor"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        // Matches squadron/bin/agent-launch: cursor-agent -p --yolo --trust
        let prompt = super::read_prompt(prompt_path)?;
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "cursor-agent".into(),
            args: vec![
                "-p".into(),
                "--yolo".into(),
                "--trust".into(),
                prompt,
            ],
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

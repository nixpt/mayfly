use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::Result;
use std::path::Path;

pub struct Opencode;

impl Adapter for Opencode {
    fn name(&self) -> &'static str {
        "opencode"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        // agent-launch used --dangerously-skip-permissions; current CLI uses --auto.
        let prompt = super::read_prompt(prompt_path)?;
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "opencode".into(),
            args: vec![
                "run".into(),
                "--format".into(),
                "json".into(),
                "--auto".into(),
                "--dir".into(),
                task.cwd.clone(),
                prompt,
            ],
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

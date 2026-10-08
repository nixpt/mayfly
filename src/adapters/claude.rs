use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::Result;
use std::path::Path;

pub struct Claude;

impl Adapter for Claude {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        // Align with squadron/bin/agent-launch claude runner: headless -p + skip perms
        // (unless tools/read_only pin an allowlist — MAYFLY-8 / MAYFLY-11).
        let prompt = super::read_prompt(prompt_path)?;
        let hatch_dir = prompt_path.parent().unwrap_or_else(|| Path::new("."));
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "claude".into(),
            args: super::claude_args(task, prompt, hatch_dir)?,
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

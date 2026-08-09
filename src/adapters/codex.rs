use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::Result;
use std::path::Path;

pub struct Codex;

impl Adapter for Codex {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        // Matches squadron/bin/agent-launch: codex exec --full-auto --skip-git-repo-check
        let prompt = super::read_prompt(prompt_path)?;
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "codex".into(),
            args: vec![
                "exec".into(),
                "--full-auto".into(),
                "--skip-git-repo-check".into(),
                prompt,
            ],
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

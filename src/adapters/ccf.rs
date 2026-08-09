use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::Result;
use std::path::Path;

/// Claude via the fleet `ccf` shape: same argv as `claude`, expects flownet env
/// (`ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN` / `FLOWNET_TOKEN_CLAUDE`).
///
/// `ccf` is a shell function; export its env before hatching, or use a login shell
/// that defines `ccf` and wrap externally. This harness just selects the claude
/// headless flags known to work under that env.
pub struct Ccf;

impl Adapter for Ccf {
    fn name(&self) -> &'static str {
        "ccf"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        let prompt = super::read_prompt(prompt_path)?;
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "claude".into(),
            args: vec![
                "-p".into(),
                prompt,
                "--output-format".into(),
                "text".into(),
                "--dangerously-skip-permissions".into(),
            ],
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

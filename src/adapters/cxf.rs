use super::{Adapter, SpawnPlan};
use crate::task::MayflyTask;
use anyhow::Result;
use std::path::Path;

/// Codex via the fleet `cxf` shape: `--profile flownet` + `FLOWNET_TOKEN_CODEX` in env.
///
/// `cxf` itself is a shell function; this adapter uses the same argv/env contract so
/// mayfly can spawn it without zsh. Caller (or smoke wrapper) must export the token.
pub struct Cxf;

impl Adapter for Cxf {
    fn name(&self) -> &'static str {
        "cxf"
    }

    fn plan(&self, task: &MayflyTask, prompt_path: &Path) -> Result<SpawnPlan> {
        let prompt = super::read_prompt(prompt_path)?;
        let profile = std::env::var("MAYFLY_CODEX_PROFILE").unwrap_or_else(|_| "flownet".into());
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "codex".into(),
            args: vec![
                "--profile".into(),
                profile,
                "exec".into(),
                "--sandbox".into(),
                "workspace-write".into(),
                "--skip-git-repo-check".into(),
                prompt,
            ],
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

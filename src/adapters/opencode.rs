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
        // Prefer OpenCode's own providers via MAYFLY_OPENCODE_MODEL
        // (e.g. opencode/big-pickle). Without -m, opencode uses its default
        // config — on this fleet that often points at flownet.
        let prompt = super::read_prompt(prompt_path)?;
        let mut args = vec![
            "run".into(),
            "--format".into(),
            "json".into(),
            "--auto".into(),
            "--dir".into(),
            task.cwd.clone(),
        ];
        // The task's own `model` wins; MAYFLY_OPENCODE_MODEL stays the fallback.
        let model = task
            .model
            .clone()
            .or_else(|| std::env::var("MAYFLY_OPENCODE_MODEL").ok())
            .filter(|m| !m.is_empty());
        if let Some(model) = model {
            args.push("-m".into());
            args.push(model);
        }
        args.push(prompt);
        Ok(SpawnPlan {
            harness: self.name().into(),
            program: "opencode".into(),
            args,
            cwd: Path::new(&task.cwd).to_path_buf(),
            prompt_path: prompt_path.to_path_buf(),
        })
    }
}

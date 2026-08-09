use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchRecord {
    pub id: String,
    pub state: HatchState,
    pub harness: String,
    pub task: String,
    pub cwd: String,
    pub ttl: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pid: Option<u32>,
    pub expire_reason: Option<String>,
    pub dir: PathBuf,
    /// Source repo for a buckets-provisioned worktree (if any).
    #[serde(default)]
    pub worktree_repo: Option<String>,
    #[serde(default)]
    pub worktree_path: Option<String>,
    #[serde(default)]
    pub worktree_branch: Option<String>,
    #[serde(default)]
    pub worktree_keep: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HatchState {
    Planned,
    Alive,
    Aging,
    Narrowing,
    Done,
    Expired,
    Failed,
}

impl std::fmt::Display for HatchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HatchState::Planned => "planned",
            HatchState::Alive => "alive",
            HatchState::Aging => "aging",
            HatchState::Narrowing => "narrowing",
            HatchState::Done => "done",
            HatchState::Expired => "expired",
            HatchState::Failed => "failed",
        };
        write!(f, "{s}")
    }
}

pub struct HatchStore {
    root: PathBuf,
}

impl HatchStore {
    pub fn open(override_dir: Option<PathBuf>) -> Result<Self> {
        let root = match override_dir {
            Some(p) => p,
            None => {
                let home = std::env::var_os("HOME").context("HOME not set")?;
                PathBuf::from(home).join(".local/state/mayfly")
            }
        };
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.root.join(id).join("record.json")
    }

    pub fn save(&self, rec: &HatchRecord) -> Result<()> {
        fs::create_dir_all(&rec.dir)?;
        let path = self.path_for(&rec.id);
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(rec)?)?;
        fs::rename(tmp, path)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<HatchRecord>> {
        let path = self.path_for(id);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&raw)?))
    }

    pub fn list(&self) -> Result<Vec<HatchRecord>> {
        let mut out = Vec::new();
        if !self.root.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            if let Some(rec) = self.get(&id)? {
                out.push(rec);
            }
        }
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(out)
    }
}

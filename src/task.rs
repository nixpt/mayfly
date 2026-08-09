use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MayflyTask {
    pub task: String,
    pub done_when: DoneWhen,
    pub harness: Harness,
    #[serde(default = "default_cwd")]
    pub cwd: String,
    #[serde(default = "default_ttl")]
    pub ttl: String,
    #[serde(default)]
    pub budget: Option<Budget>,
    #[serde(default)]
    pub aging: Option<Aging>,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub constraints: Constraints,
}

fn default_cwd() -> String {
    ".".into()
}
fn default_ttl() -> String {
    "15m".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DoneWhen {
    Command {
        run: String,
        #[serde(default)]
        expect_exit: i32,
    },
    FilesExist {
        paths: Vec<String>,
    },
}

impl DoneWhen {
    pub fn summary(&self) -> String {
        match self {
            DoneWhen::Command { run, expect_exit } => {
                format!("command exit={expect_exit}: {run}")
            }
            DoneWhen::FilesExist { paths } => format!("files_exist: {}", paths.join(", ")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Harness {
    Claude,
    /// Claude under fleet `ccf` env (flownet Anthropic-compat).
    Ccf,
    Cursor,
    Codex,
    /// Codex under fleet `cxf` shape (`--profile flownet` + FLOWNET_TOKEN_CODEX).
    Cxf,
    Cece,
    Opencode,
    Exec,
}

impl std::fmt::Display for Harness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Harness::Claude => "claude",
            Harness::Ccf => "ccf",
            Harness::Cursor => "cursor",
            Harness::Codex => "codex",
            Harness::Cxf => "cxf",
            Harness::Cece => "cece",
            Harness::Opencode => "opencode",
            Harness::Exec => "exec",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Budget {
    pub max_turns: Option<u32>,
    pub max_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aging {
    #[serde(default = "default_warn")]
    pub warn_at: f64,
    #[serde(default = "default_narrow")]
    pub narrow_at: f64,
}

fn default_warn() -> f64 {
    0.5
}
fn default_narrow() -> f64 {
    0.75
}

impl Default for Aging {
    fn default() -> Self {
        Self {
            warn_at: default_warn(),
            narrow_at: default_narrow(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    #[serde(default = "default_true")]
    pub no_spawn: bool,
    #[serde(default = "default_protected_branches")]
    pub no_commit_to: Vec<String>,
    #[serde(default)]
    pub paths_allow: Vec<String>,
}

fn default_true() -> bool {
    true
}
fn default_protected_branches() -> Vec<String> {
    vec!["main".into(), "master".into(), "dev".into()]
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            no_spawn: true,
            no_commit_to: default_protected_branches(),
            paths_allow: vec![],
        }
    }
}

//! Hatch-local Claude `--mcp-config` from the user's `~/.claude.json` (MAYFLY-11).
//!
//! Env values from server configs stay in the generated file only (mode 0600).
//! Callers must tear the file down after the hatch; never log its contents.

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const MCP_CONFIG_FILE: &str = "mcp-config.json";

/// Override with `MAYFLY_CLAUDE_CONFIG` (tests); else `$HOME/.claude.json`.
pub fn claude_config_path() -> PathBuf {
    if let Some(p) = std::env::var_os("MAYFLY_CLAUDE_CONFIG") {
        return PathBuf::from(p);
    }
    let home = std::env::var_os("HOME").unwrap_or_default();
    PathBuf::from(home).join(".claude.json")
}

pub fn mcp_config_path(hatch_dir: &Path) -> PathBuf {
    hatch_dir.join(MCP_CONFIG_FILE)
}

/// Base tool name: `Bash(gh pr list:*)` → `Bash`, `mcp__mailgate__mail_read` → that string.
pub fn tool_base_name(spec: &str) -> &str {
    spec.split('(').next().unwrap_or(spec)
}

/// `mcp__<server>__…` → `Some(server)`.
pub fn mcp_server_from_tool(spec: &str) -> Option<&str> {
    let base = tool_base_name(spec);
    let rest = base.strip_prefix("mcp__")?;
    let server = rest.split("__").next()?;
    if server.is_empty() {
        None
    } else {
        Some(server)
    }
}

fn load_user_mcp_servers() -> Result<Map<String, Value>> {
    let path = claude_config_path();
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read Claude config {}", path.display()))?;
    let v: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse Claude config {}", path.display()))?;
    let Some(obj) = v.get("mcpServers").and_then(|x| x.as_object()) else {
        bail!(
            "Claude config {} has no top-level mcpServers object",
            path.display()
        );
    };
    Ok(obj.clone())
}

/// Ensure every name exists in the user's Claude `mcpServers` (no file write).
pub fn validate_mcp_server_names(names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    let all = load_user_mcp_servers()?;
    for name in names {
        if !all.contains_key(name) {
            bail!("unknown mcp server '{name}' (not in user Claude mcpServers)");
        }
    }
    Ok(())
}

/// Write a 0600 `--mcp-config` containing only the named servers. Returns its path.
pub fn write_mcp_config(hatch_dir: &Path, names: &[String]) -> Result<PathBuf> {
    let all = load_user_mcp_servers()?;
    let mut selected = Map::new();
    for name in names {
        let Some(cfg) = all.get(name) else {
            bail!("unknown mcp server '{name}' (not in user Claude mcpServers)");
        };
        selected.insert(name.clone(), cfg.clone());
    }
    let path = mcp_config_path(hatch_dir);
    let doc = json!({ "mcpServers": selected });
    let bytes = serde_json::to_vec_pretty(&doc)?;
    write_private(&path, &bytes)?;
    Ok(path)
}

pub fn remove_mcp_config(hatch_dir: &Path) {
    let _ = fs::remove_file(mcp_config_path(hatch_dir));
}

/// RAII: delete hatch-local mcp-config on drop (secrets must not linger).
pub struct McpConfigGuard(PathBuf);

impl McpConfigGuard {
    pub fn for_hatch(hatch_dir: &Path) -> Self {
        Self(mcp_config_path(hatch_dir))
    }
}

impl Drop for McpConfigGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("create {}", path.display()))?;
        f.write_all(bytes)
            .with_context(|| format!("write {}", path.display()))?;
        f.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialise tests that touch MAYFLY_CLAUDE_CONFIG.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_claude_config(body: &str, f: impl FnOnce(PathBuf)) {
        let _g = ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!(
            "mayfly-mcp-cfg-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).unwrap();
        let cfg = dir.join("claude.json");
        fs::write(&cfg, body).unwrap();
        let prev = std::env::var_os("MAYFLY_CLAUDE_CONFIG");
        std::env::set_var("MAYFLY_CLAUDE_CONFIG", &cfg);
        f(dir.clone());
        match prev {
            Some(v) => std::env::set_var("MAYFLY_CLAUDE_CONFIG", v),
            None => std::env::remove_var("MAYFLY_CLAUDE_CONFIG"),
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn parses_mcp_server_from_tool() {
        assert_eq!(
            mcp_server_from_tool("mcp__mailgate__mail_read"),
            Some("mailgate")
        );
        assert_eq!(mcp_server_from_tool("Bash(gh pr list:*)"), None);
        assert_eq!(mcp_server_from_tool("Write(/tmp/x)"), None);
    }

    #[test]
    fn writes_only_named_servers_at_0600_and_teardown_removes() {
        with_claude_config(
            r#"{"mcpServers":{"mailgate":{"type":"stdio","command":"mg","env":{"SECRET":"s"}},"other":{"type":"stdio","command":"o"}}}"#,
            |dir| {
                let hatch = dir.join("hatch");
                fs::create_dir_all(&hatch).unwrap();
                let path = write_mcp_config(&hatch, &["mailgate".into()]).unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
                    assert_eq!(mode, 0o600);
                }
                let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
                let servers = v["mcpServers"].as_object().unwrap();
                assert!(servers.contains_key("mailgate"));
                assert!(!servers.contains_key("other"));
                assert_eq!(servers["mailgate"]["env"]["SECRET"], "s");
                remove_mcp_config(&hatch);
                assert!(!path.exists());
            },
        );
    }

    #[test]
    fn unknown_server_name_errors() {
        with_claude_config(
            r#"{"mcpServers":{"mailgate":{"type":"stdio","command":"mg"}}}"#,
            |_| {
                let err = validate_mcp_server_names(&["nope".into()]).unwrap_err();
                assert!(err.to_string().contains("unknown mcp server 'nope'"));
            },
        );
    }

    #[test]
    fn guard_removes_on_drop() {
        with_claude_config(
            r#"{"mcpServers":{"mailgate":{"type":"stdio","command":"mg"}}}"#,
            |dir| {
                let hatch = dir.join("hatch");
                fs::create_dir_all(&hatch).unwrap();
                let path = write_mcp_config(&hatch, &["mailgate".into()]).unwrap();
                {
                    let _g = McpConfigGuard::for_hatch(&hatch);
                    assert!(path.exists());
                }
                assert!(!path.exists());
            },
        );
    }
}

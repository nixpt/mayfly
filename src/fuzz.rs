//! Fuzziness gate — reject vague / unbounded tasks before spawn.

use crate::task::MayflyTask;
use crate::ttl::parse_ttl;
use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FuzzError {
    #[error("vague verb without a bound: {0}")]
    VagueVerb(String),
    #[error("multiple top-level goals (split into separate hatches)")]
    MultipleGoals,
    #[error("task lacks a concrete anchor (file, test, function, error, or path)")]
    NoConcreteAnchor,
    #[error("ttl {0} exceeds mayfly max (2h) — use a horse")]
    TtlTooLong(String),
    #[error("invalid ttl: {0}")]
    BadTtl(String),
}

static VAGUE: OnceLock<Regex> = OnceLock::new();
static MULTI: OnceLock<Regex> = OnceLock::new();
static ANCHOR: OnceLock<Regex> = OnceLock::new();

fn vague_re() -> &'static Regex {
    VAGUE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(improve|polish|rethink|clean\s*up|make\s+better|enhance|optimize\s+generally|revamp|overhaul)\b",
        )
        .expect("vague regex")
    })
}

fn multi_re() -> &'static Regex {
    MULTI.get_or_init(|| {
        Regex::new(r"(?i)\b(and also|then also|as well as|; and)\b").expect("multi regex")
    })
}

fn anchor_re() -> &'static Regex {
    ANCHOR.get_or_init(|| {
        Regex::new(
            r"(?i)(\.rs|\.ts|\.tsx|\.js|\.py|\.go|\.md|\.toml|\.json|/|::|test_|#\d+|error:|FAIL|panic|bug)",
        )
        .expect("anchor regex")
    })
}

pub fn check(task: &MayflyTask) -> Result<(), FuzzError> {
    let text = task.task.trim();

    if let Some(m) = vague_re().find(text) {
        if !anchor_re().is_match(text) {
            return Err(FuzzError::VagueVerb(m.as_str().to_string()));
        }
    }

    if multi_re().is_match(text) {
        return Err(FuzzError::MultipleGoals);
    }

    if !anchor_re().is_match(text) {
        return Err(FuzzError::NoConcreteAnchor);
    }

    let dur = parse_ttl(&task.ttl).map_err(|e| FuzzError::BadTtl(e.to_string()))?;
    if dur > std::time::Duration::from_secs(2 * 60 * 60) {
        return Err(FuzzError::TtlTooLong(task.ttl.clone()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{DoneWhen, Harness, MayflyTask};

    fn task(s: &str, ttl: &str) -> MayflyTask {
        MayflyTask {
            task: s.into(),
            done_when: DoneWhen::Command {
                run: "true".into(),
                expect_exit: 0,
            },
            harness: Harness::Exec,
            cwd: ".".into(),
            ttl: ttl.into(),
            budget: None,
            aging: None,
            artifacts: vec![],
            constraints: Default::default(),
        }
    }

    #[test]
    fn accepts_concrete_fix() {
        assert!(check(&task(
            "Fix the failing unit test in foo::bar::test_baz so cargo test passes",
            "15m"
        ))
        .is_ok());
    }

    #[test]
    fn rejects_vague_multi_goal() {
        let t = task(
            "Improve the codebase and also make the docs feel better",
            "3h",
        );
        assert!(check(&t).is_err());
    }

    #[test]
    fn rejects_long_ttl() {
        let t = task("Fix bug in src/main.rs panic on empty input", "3h");
        assert!(matches!(check(&t), Err(FuzzError::TtlTooLong(_))));
    }
}

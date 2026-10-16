# MAYFLY-16 — `copilot` harness: GitHub Copilot CLI adapter

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-16 |
| **Priority** | P2 |
| **Status** | Done |
| **Phase** | M5 |
| **Assignee** | vega |
| **Dependencies** | none (sibling of MAYFLY-5, which stays open) |
| **Estimated effort** | S |

## Problem

GitHub Models (the free OpenAI-shaped endpoint) was retired on 2026-07-30, but the GitHub Copilot CLI
(`copilot -p`) runs headless with a GitHub token, including a Codespace's own `GITHUB_TOKEN`. mayfly had no
harness for it. This is the **local CLI**, an ordinary `Child` process that fits the existing adapter trait.
MAYFLY-5 (Copilot **cloud** agent, no local PID) is a different shape and stays open.

## Success criteria

- [x] `harness: "copilot"` spawns `copilot -p <task> -s`.
- [x] `model` → `--model`; `read_only` enforced; `tools` translated into Copilot permission rules or refused;
      `mcp_servers` and `budget.max_usd` refused. No knob is silently ignored or broadened.
- [x] Live hatches succeed end to end, and read-only is shown to block writes.

## Technical approach

- `src/adapters/copilot.rs`: argv + `tool_rules()` (Claude-syntax `tools` → `shell(...)` / `write(/abs)` /
  `--add-dir`, refusing globs, scoped reads, bare relative `Edit(file)`, MCP and other tools).
- `src/adapters/mod.rs`: `takes_model`, `has_read_only`, `supports_tools` (tools without `mcp_servers`),
  copilot validation, and an `unenforced` note for Copilot's auto-approved read-only shell commands.
- Schema enum + descriptions, README tables, `examples/fleet/pr-reviewer-copilot.json`.

## Resolution (2026-10-16)

**Permission model, probed live** (Copilot CLI 1.0.98, `-p` mode, GitHub Codespace):
- A tool that needs approval is denied, never prompted, so a hatch can't hang.
- `--deny-tool` beats every allow rule, including `--allow-all-tools`.
- `write(/abs/file)` allows exactly that file; other paths are denied.
- With no allow rules, Copilot still auto-approves its built-in read-only commands: `ls`, `cat`, `pwd`, `date`,
  `git log` and `gh pr list` ran; `env`, `curl`, `python3`, `touch`, `rm` and `git commit` were denied. Hence
  `--deny-tool shell` when `tools` has no `Bash` rule, and a `validate` note when it does.

**Live hatches** (`mayfly` from this branch, in a 4-core Codespace, auth = the Codespace `GITHUB_TOKEN`):

| Hatch | Mode | Result |
|---|---|---|
| PR reviewer (`gh pr view/diff` + reads → review file) on nixpt/crush-ast#263 | `tools` | ✅ done; request-changes verdict with file:line evidence, same main finding as a manual review |
| Test writer (add a unit test to `src/task.rs`) with `--worktree` (buckets) | full access | ✅ done; `done_when` ran the new test, worktree torn down |
| PR survey (fleet `pr-survey.json` on copilot) | `tools` | ✅ done; accurate tables for 3 repos |
| Read-only reviewer told to write `RO_PROBE.md` | `read_only` | ✅ correctly **failed**: "write operations were denied", no file, clean `git status` |

The three parallel hatches finished in 58 s. `validate` refuses `mcp_servers`, `budget.max_usd` and `Edit(src/**)`
on copilot (exit 2).

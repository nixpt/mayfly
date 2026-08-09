# MAYFLY-1 — kitchen / worktree integration

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-1 |
| **Priority** | P2 |
| **Status** | Backlog |
| **Phase** | M1 |
| **Assignee** | unassigned |
| **Dependencies** | none |
| **Estimated effort** | M |

## Problem

v0 hatches into whatever `cwd` the task provides. Shared source checkouts are unsafe under mom's kitchen. Callers should be able to ask mayfly to provision an ephemeral worktree (via `kitchen` / `buckets worktree`) and clean it up on expire.

## Success criteria

- [ ] `mayfly hatch task.json --kitchen` provisions an isolated worktree before spawn
- [ ] Default path refuses (or strongly warns on) hatching into a non-worktree git checkout of a known shared root
- [ ] Teardown removes the worktree on done/expire/fail
- [ ] `cargo test` covers the refuse/warn path without requiring a live kitchen binary when mocked

## Technical approach

- Detect `kitchen` / `buckets worktree` on PATH; feature-gate behavior if missing.
- Add CLI flag + task field (`isolation: kitchen | cwd`).
- Record worktree path on the hatch record for `expire` cleanup.

## Files to modify

- `src/main.rs` — `--kitchen` flag
- `src/hatch.rs` — provision + teardown
- `DESIGN.md` — document v1 isolation

## Non-goals

- Replacing kitchen itself
- Multi-repo path-dep rewriting

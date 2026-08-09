# MAYFLY-4 — remote + release tags

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-4 |
| **Priority** | P3 |
| **Status** | Done |
| **Phase** | M4 |
| **Assignee** | cursor |
| **Dependencies** | none |
| **Estimated effort** | S |

## Problem

Need public remote + tags so `scripts/bump-version.sh` / release workflow can run.

## Success criteria

- [x] GitHub remote exists — https://github.com/nixpt/mayfly (public)
- [x] `v0.1.0` + `v0.1.1` tags on main
- [x] GitHub Releases for both tags
- [x] Fleet usage note in README (`mayfly hatch … --worktree`)

## Resolution

Remote + tags landed 2026-08-09. Latest: **v0.1.1** (live adapters + smoke). Fleet usage under README "Fleet usage (horse → mayfly)".

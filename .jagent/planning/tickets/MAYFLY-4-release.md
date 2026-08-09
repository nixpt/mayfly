# MAYFLY-4 — remote + v0.1.0 tag

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

Need public remote + initial tag so `scripts/bump-version.sh` / release workflow can run.

## Success criteria

- [x] GitHub remote exists — https://github.com/nixpt/mayfly (public)
- [x] `v0.1.0` tag on main
- [x] `BUMPVER_DRY_RUN=1` plans patch/minor correctly after tag
- [x] Fleet usage note in README (`mayfly hatch … --worktree`)

## Resolution

Remote + tag landed 2026-08-09. Fleet usage documented under README "Fleet usage (horse → mayfly)".

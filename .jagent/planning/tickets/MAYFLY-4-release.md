# MAYFLY-4 — remote + v0.1.0 tag

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-4 |
| **Priority** | P3 |
| **Status** | Backlog |
| **Phase** | M4 |
| **Assignee** | unassigned |
| **Dependencies** | none |
| **Estimated effort** | S |

## Problem

Repo is local-only. `scripts/bump-version.sh` + release workflow need a remote and an initial `v0.1.0` tag before automatic bumps can run.

## Success criteria

- [ ] GitHub remote exists (private OK)
- [ ] Initial commit(s) on `main` with `v0.1.0` tag
- [ ] `BUMPVER_DRY_RUN=1 ./scripts/bump-version.sh` shows a sensible plan after a dummy feat commit
- [ ] Short fleet usage note in README (how a horse calls `mayfly hatch`)

## Technical approach

- `gh repo create` (or captain-approved remote)
- Tag v0.1.0 by hand once (per bump-version docs)
- Keep package version at 0.1.0 until first post-tag bump

## Files to modify

- `README.md`
- git remotes / tags only

## Non-goals

- crates.io publish

# MAYFLY-1 — buckets worktree isolation

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-1 |
| **Priority** | P2 |
| **Status** | Done |
| **Phase** | M1 |
| **Assignee** | cursor |
| **Dependencies** | none |
| **Estimated effort** | M |

## Problem

v0 hatched into whatever `cwd` the task provided. Shared source checkouts are unsafe under mom's kitchen.

## Success criteria

- [x] `mayfly hatch task.json --worktree <repo>` provisions via `buckets worktree create`
- [x] Teardown via `buckets worktree remove --force` on done/expire/fail
- [x] `--keep-worktree` / `--branch` / `--from` flags
- [x] Warn when cwd looks like a primary source checkout without `--worktree`
- [x] No path-dep on buckets; no flame/firefly wiring

## Resolution

Shipped `src/worktree.rs` + hatch/CLI flags. Uses public `buckets` CLI only.
Flame/firefly/flare held deliberately (heat/brands/fuel layer, not git worktree).

## Non-goals

- kitchen enter/ship UX (horse lifecycle)
- firefly spark wrapping the harness

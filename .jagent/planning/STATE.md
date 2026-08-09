# Planning state — mayfly

**Updated:** 2026-08-09
**Milestone focus:** M2 done → M3 aging inject (MAYFLY-2)
**Branch:** `agent/cursor/MAYFLY-3`

## Delivery snapshot

| Track | Status | Notes |
|-------|--------|--------|
| CLI surface | **shipped** | validate / hatch / status / expire / list |
| buckets worktree | **shipped** | MAYFLY-1 |
| Live adapters | **shipped** | MAYFLY-3 — cursor OK; claude/codex honest auth fails |
| Release | **shipped** | MAYFLY-4 — public remote + v0.1.0 |
| Aging inject | **open** | MAYFLY-2 |
| flame/firefly | **held** | |

## Active work

Closing MAYFLY-3. Next open: MAYFLY-2 (aging inject).

## Blockers

Claude OAuth / Codex API key on this box — not a mayfly bug; smokes fail honestly.

## Metrics

- Unit tests: 4 passing
- Smoke: cursor OK; claude OAuth expired; codex 401

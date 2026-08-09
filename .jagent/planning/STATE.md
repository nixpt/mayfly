# Planning state — mayfly

**Updated:** 2026-08-09
**Milestone focus:** v0.1.1 released; optional MAYFLY-2 (aging inject) parked
**Branch:** `main` @ origin · tag `v0.1.1`

## Delivery snapshot

| Track | Status | Notes |
|-------|--------|--------|
| CLI + fuzz + TTL | **shipped** | v0.1.0 → v0.1.1 |
| buckets `--worktree` | **shipped** | MAYFLY-1 |
| Live adapters | **shipped** | MAYFLY-3 — cursor/ccf/cxf/opencode/exec smoked OK |
| Public remote + tags | **shipped** | MAYFLY-4 · https://github.com/nixpt/mayfly |
| Aging inject | **parked** | MAYFLY-2 — post-release polish, not blocking |
| flame/firefly | **held** | wrong layer |

## Active work

Docs sync after v0.1.1. No open P0/P1 tickets.

## Blockers

_None._ Provider auth/budget failures are environment, not product bugs.

## Metrics

- Unit tests: 4 passing
- Release: v0.1.1 (GitHub Release + `scripts/bump-version.sh`)
- Smoke OK: cursor, ccf, cxf, opencode (`big-pickle`), exec

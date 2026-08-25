# Planning state — mayfly

**Updated:** 2026-08-25
**Milestone focus:** **M5 — local lifecycle reliability**
**Branch:** `agent/buffy/MAYFLY-6` from `origin/main`

## Delivery snapshot

| Track | Status | Notes |
|-------|--------|-------|
| CLI + fuzz + TTL | **shipped** | v0.1.0+ |
| buckets `--worktree` | **shipped** | MAYFLY-1 |
| Live adapters | **shipped** | MAYFLY-3 |
| Public remote + tags | **shipped** | MAYFLY-4; current main is v0.1.4 |
| Aging inject | **parked** | MAYFLY-2 — optional, no owner |
| Local lifecycle reliability | **in progress** | MAYFLY-6; output draining, process groups, cleanup guard |
| Contract parity | **next** | MAYFLY-7 |
| Copilot cloud-agent adapter | **design backlog** | MAYFLY-5; remote-handle design follows local contract work |
| flame/firefly | **held** | wrong layer |

## Active work

MAYFLY-6 on `agent/buffy/MAYFLY-6`.

## Blockers

The real-worktree integration fixture still needs a provider test before MAYFLY-6 can be marked Done. Unit coverage and release build are green.

## Metrics

- Unit tests: 7 passing on the MAYFLY-6 branch
- Latest main release: v0.1.4
- Smoke baseline: cursor, ccf, cxf, opencode (`MAYFLY_OPENCODE_MODEL`), exec

# Planning state — mayfly

**Updated:** 2026-08-09
**Milestone focus:** M1 done → M2 live adapters
**Branch:** `main` @ origin (nixpt/mayfly public)

## Delivery snapshot

| Track | Status | Notes |
|-------|--------|--------|
| CLI surface | **shipped** | validate / hatch / status / expire / list |
| Task schema | **shipped** | `schemas/task.schema.json` + `MayflyTask` |
| Fuzziness gate | **shipped** | vague verbs / multi-goal / TTL>2h rejected |
| TTL + aging states | **shipped** | alive → aging → narrowing → expired |
| Adapters | **partial** | exec + cursor live-tested; claude/codex argv only |
| Rename / IP scrub | **shipped** | mayfly tone |
| `.dejavue` / `.jagent` / bump-version | **shipped** | |
| buckets worktree isolation | **shipped** | MAYFLY-1 — `--worktree` / `--keep-worktree` |
| flame/firefly | **held** | different layer; do not duplicate |
| Live claude/codex smoke | **open** | MAYFLY-3 |

## Active work

Next: MAYFLY-3 (claude/codex live), MAYFLY-4 (v0.1.0 tag).

## Blockers

_None known._

## Metrics

- Unit tests: 4 passing
- Manual: exec, TTL expire, cursor hatch, buckets worktree e2e verified

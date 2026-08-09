# Planning state — mayfly

**Updated:** 2026-08-09
**Milestone focus:** M0 complete locally → M1 isolation
**Branch:** local `main` (unpushed scaffold; no remote yet)

## Delivery snapshot

| Track | Status | Notes |
|-------|--------|--------|
| CLI surface | **shipped** | validate / hatch / status / expire / list |
| Task schema | **shipped** | `schemas/task.schema.json` + `MayflyTask` |
| Fuzziness gate | **shipped** | vague verbs / multi-goal / TTL>2h rejected |
| TTL + aging states | **shipped** | alive → aging → narrowing → expired |
| Adapters | **partial** | exec live-tested; claude/cursor/codex argv only |
| Rename / IP scrub | **shipped** | mayfly tone; no Meeseeks branding |
| `.dejavue` / `.jagent` / bump-version | **shipped** | this session |
| Kitchen integration | **open** | MAYFLY-1 |
| Live harness smoke | **open** | MAYFLY-3 |

## Active work

Scaffold complete. Next open tickets: MAYFLY-1 (`--kitchen`), MAYFLY-3 (live adapters).

## Blockers

_None known._ Remote + initial `v0.1.0` tag not created yet (MAYFLY-4).

## Metrics

- Unit tests: 4 passing
- Manual: exec success, exec fail, TTL expire (1s → exit 124) verified

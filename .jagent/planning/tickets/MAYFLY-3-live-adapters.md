# MAYFLY-3 — live harness adapter smoke

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-3 |
| **Priority** | P2 |
| **Status** | Done |
| **Phase** | M2 |
| **Assignee** | cursor |
| **Dependencies** | none |
| **Estimated effort** | M |

## Problem

`claude` / `cursor` / `codex` adapters only had argv sketches. Flags drift; dry-run is not enough.

## Success criteria

- [x] One tiny fixture task under each available harness (succeed or fail honestly)
- [x] Missing binary yields a clear error (not a hang)
- [x] DESIGN / README note which harnesses were last verified

## Resolution (2026-08-09)

- Fixture: `examples/smoke-touch-file.json` + `scripts/smoke-harness.sh`
- Claude: added `--dangerously-skip-permissions` (agent-launch parity)
- Codex: `--full-auto` → `--sandbox workspace-write` (deprecation)
- Cursor: unchanged; smoke **OK**
- Claude/Codex: argv launches cleanly; failed on auth (OAuth / 401) — honest failure with exit 1
- `adapters::spawn` checks PATH and errors clearly

## Non-goals

- Full fleet dispatch integration (MAYFLY-4 leftovers: fleet usage note in README)

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
- Added/verified: `ccf`, `cxf`, `opencode` (+ `MAYFLY_OPENCODE_MODEL`)
- Claude: `--dangerously-skip-permissions`; prefer fleet `ccf`
- Codex: `--sandbox workspace-write`; prefer fleet `cxf`
- Smoke **OK**: cursor, ccf, cxf, opencode, exec
- Raw claude/codex/cece: fail honestly on auth/budget when env missing
- `adapters::spawn` checks PATH and errors clearly

## Non-goals

- Full aging inject (MAYFLY-2)

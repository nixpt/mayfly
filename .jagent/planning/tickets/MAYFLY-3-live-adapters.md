# MAYFLY-3 — live harness adapter smoke

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-3 |
| **Priority** | P2 |
| **Status** | Backlog |
| **Phase** | M2 |
| **Assignee** | unassigned |
| **Dependencies** | none |
| **Estimated effort** | M |

## Problem

`claude` / `cursor` / `codex` adapters only have argv sketches matching `agent-launch`. They have not been live-validated on this box. Flags drift; dry-run is not enough.

## Success criteria

- [ ] One tiny fixture task succeeds (or fails honestly) under each available harness
- [ ] Missing binary yields a clear error (not a hang)
- [ ] DESIGN / README note which harnesses were last verified and on which version

## Technical approach

- Add `examples/live-touch-file.json` style fixture
- Document `MAYFLY_SMOKE_HARNESS=claude mayfly hatch …` for CI-optional smoke
- Align argv with current `squadron/bin/agent-launch` runner table

## Files to modify

- `src/adapters/*.rs`
- `examples/`
- `README.md`

## Non-goals

- Full fleet dispatch integration (MAYFLY-4)

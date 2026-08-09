# MAYFLY-2 — aging / narrowing inject

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-2 |
| **Priority** | P3 |
| **Status** | Backlog |
| **Phase** | M3 |
| **Assignee** | unassigned |
| **Dependencies** | MAYFLY-3 |
| **Estimated effort** | M |

## Problem

Aging ladder currently only flips record state. DESIGN calls for injecting "narrow to done_when" into the live harness when lifespan crosses warn/narrow thresholds. Most harnesses have no stdin inject — need best-effort per adapter.

## Success criteria

- [ ] At `warn_at`, adapter gets an optional inject (or sidecar note file) asking to narrow scope
- [ ] At `narrow_at`, writes outside `paths_allow` are discouraged/enforced where feasible
- [ ] Adapters that cannot inject degrade silently (state still flips)
- [ ] Unit/integration coverage for state transitions remains green

## Technical approach

- Extend `Adapter` trait with `inject(message) -> Result<()>`
- Prefer file-drop (`MAYFLY_NARROW`) for harnesses that poll cwd; stdin only where supported

## Files to modify

- `src/adapters/mod.rs` + per-harness
- `src/hatch.rs` — call inject on threshold cross

## Non-goals

- Guaranteed compliance from the LLM
- Killing tools mid-flight except on TTL

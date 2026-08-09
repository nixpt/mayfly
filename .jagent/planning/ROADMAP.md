# Roadmap — mayfly

Living plan. Dejavue holds *why*; this file holds *sequence*.

## North star

A harness-agnostic launcher for ephemeral single-purpose agents: validate a concrete task, hatch into Cursor/Claude/Codex/OpenCode/…, watch a mechanical `done_when` + hard TTL, tear down, leave only artifacts.

## Current phase: released (v0.1.1)

M0–M2 + fleet docs are in **v0.1.1**. Next work is optional polish (MAYFLY-2 aging inject), not a release gate.

---

## Milestones

| Phase | Name | Goal | Exit criteria |
|-------|------|------|----------------|
| **M0** | Foundation | CLI, schema, fuzz gate, adapters, TTL | `cargo test` + exec hatch + TTL expire ✅ |
| **M1** | Isolation | `--worktree` via public buckets | create → hatch → remove --force ✅ |
| **M2** | Live adapters | Real harness end-to-end | Smoke script + matrix in README ✅ |
| **M3** | Aging inject | Warn/narrow messages into live sessions | Best-effort inject (MAYFLY-2, parked) |
| **M4** | Fleet wire | Documented use from foreman/horses | README fleet section ✅ |

---

## Non-goals (standing)

- Personas, scrolls, bridge DMs (that's horses)
- Multi-day refactors / open-ended "improve X"
- Recursive hatch from inside a mayfly
- Replacing `agent-launch` for durable fleet work
- Flame/firefly/flare integration (different layer)

## Version tags

| Tag | Maps to |
|-----|---------|
| v0.1.0 | Initial public scaffold + `--worktree` |
| v0.1.1 | Live adapters (ccf/cxf/opencode/…), smoke harness, docs |
| later | MAYFLY-2 aging inject if/when needed |

# Roadmap — mayfly

Living plan. Dejavue holds *why*; this file holds *sequence*.

## North star

A harness-agnostic launcher for ephemeral single-purpose agents: validate a concrete task, hatch into Cursor/Claude/Codex/…, watch a mechanical `done_when` + hard TTL, tear down, leave only artifacts.

## Current phase: M0 foundation (shipped locally)

CLI + schema + fuzz gate + adapters + TTL watch are green on this box. Next is isolation integration and live harness validation.

---

## Milestones

| Phase | Name | Goal | Exit criteria |
|-------|------|------|----------------|
| **M0** | Foundation | CLI, schema, fuzz gate, adapters, TTL | `cargo test` + exec hatch + TTL expire ✅ |
| **M1** | Isolation | `--worktree` via public buckets | create → hatch → remove --force ✅ |
| **M2** | Live adapters | Real cursor/claude/codex end-to-end | One smoke task per harness green |
| **M3** | Aging inject | Warn/narrow messages into live sessions | Best-effort inject without breaking adapters |
| **M4** | Fleet wire | Documented use from foreman/horses | Example dispatch + state dir conventions |

---

## Non-goals (standing)

- Personas, scrolls, bridge DMs (that's horses)
- Multi-day refactors / open-ended "improve X"
- Recursive hatch from inside a mayfly
- Replacing `agent-launch` for durable fleet work

## Version tags (when releasing)

| Tag | Maps to |
|-----|---------|
| v0.1.0 | M0 complete (current local state) |
| v0.2.0 | M1 isolation |
| v0.3.0 | M2 live adapters |

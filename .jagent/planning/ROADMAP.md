# Roadmap — mayfly

Living plan. Dejavue holds *why*; this file holds *sequence*.

## North star

A harness-agnostic launcher for ephemeral single-purpose agents: validate a concrete task, hatch into Cursor/Claude/Codex/OpenCode/…, watch a mechanical `done_when` + hard TTL, tear down, leave only artifacts.

## Current phase: M5 local reliability

The foundation arc is closed. The next gate is making local hatches reliable
under noisy output, descendant processes, and failure paths before introducing a
remote execution backend.

---

## Milestones

| Phase | Name | Goal | Exit criteria |
|-------|------|------|---------------|
| **M0** | Foundation | CLI, schema, fuzz gate, adapters, TTL | `cargo test` + exec hatch + TTL expire ✅ |
| **M1** | Isolation | `--worktree` via public buckets | create → hatch → remove --force ✅ |
| **M2** | Live adapters | Real harness end-to-end | Smoke script + matrix in README ✅ |
| **M3** | Aging inject | Warn/narrow messages into live sessions | Best-effort inject (MAYFLY-2, parked) |
| **M4** | Fleet wire | Documented use from foreman/horses | README fleet section ✅ |
| **M5** | Local reliability | No pipe deadlocks, process leaks, or missed worktree cleanup | Large-output, expiry, error-path, and worktree integration tests |
| **M6** | Contract parity | Align schema, DESIGN, and runtime behavior | Every documented task field is implemented or explicitly removed |
| **M7** | Remote design | Generalize execution handles for non-local workers | MAYFLY-5 ADR, remote task example, auth/expiry semantics |
| **M8** | Remote MVP | Add a bounded remote adapter behind the agreed interface | Mocked backend plus authenticated disposable-repo smoke |
| **M9** | Fleet release | Publish machine-readable reports and v0.2 guidance | CI matrix, fleet examples, updated docs and release |

## Next tickets

- **MAYFLY-6** — local lifecycle reliability; current branch `agent/buffy/MAYFLY-6`.
- **MAYFLY-7** — task schema/runtime contract parity; depends on MAYFLY-6.
- **MAYFLY-5** — GitHub Copilot cloud-agent design spike; depends on the local contract and should precede remote implementation.

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
| v0.1.1 | Live adapters (ccf/cxf/opencode/…), smoke harness |
| v0.1.2 | Docs/dejavue sync; foundation arc closed |
| v0.1.3 | Release metadata and public maintenance |
| v0.1.4 | Current main release before M5 reliability work |
| later | MAYFLY-2 aging inject if a concrete harness needs it |

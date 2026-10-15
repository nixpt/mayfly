# MAYFLY-15 — retire the `cece` harness

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-15 |
| **Priority** | P3 |
| **Status** | Open (after MAYFLY-14) |
| **Assignee** | — |
| **Dependencies** | MAYFLY-14 (`bro` harness) |
| **Estimated effort** | S |

## Problem
The fleet is retiring the cece line (squadron SQ-242, captain s481). cece-rs has had no commits since 2026-08-11. bro replaces it as the fleet's own harness. mayfly's `cece` harness (`src/adapters/cece.rs`, `Harness::Cece`, `schemas/task.schema.json`) runs `cece-rs`, which is being uninstalled.

## Scope
- Remove `Harness::Cece`, `src/adapters/cece.rs`, the schema enum value, the DESIGN.md / README rows and the smoke-matrix row.
- A task file with `"harness": "cece"` fails validation with a message pointing at `bro`.
- Release note (removed harness).

## Done when
Tests pass, the schema rejects `cece` with the bro hint, and the docs have no cece rows.

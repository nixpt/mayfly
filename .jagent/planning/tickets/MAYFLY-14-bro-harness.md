# MAYFLY-14 — `bro` harness

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-14 |
| **Priority** | P2 |
| **Status** | Open |
| **Assignee** | — |
| **Dependencies** | bro-cli BRO-134 (`bro exec` engine flags + claude-compatible record) |
| **Estimated effort** | S |

## Problem

mayfly hatches single-shot agents on `claude`, `ccf`, `cursor`, `codex`, `cxf`, `cece` and `opencode`, but not **bro** (`nixpt/bro-cli`). bro on a local or zorro model through flownet is the fleet's no-cost engine. Its `bro exec` is already a one-shot runner, and BRO-134 makes it engine-shaped.

## Scope

- Add `Harness::Bro` (serde `"bro"`). argv: `bro exec --cwd <cwd> --sandbox process --max-turns <budget.max_turns> [-m <MAYFLY_BRO_MODEL>] <prompt>`.
- Pick up `total_cost_usd` / `usage` from the record where mayfly reads harness usage.
- `--sandbox process`, because `cvm` (bro's interactive default) has no network.
- `MAYFLY_BRO_BIN` override, the same as other harness bins.
- `no_spawn` constraint: pass bro's tool profile `coding`, which has no dispatch tools. Or use `BRO_DISABLED_TOOLS=cowboy_dispatch,barn_run,task,delegate_task,send_message`.
- Add a DESIGN.md harness table row and a README smoke-matrix row. `./scripts/smoke-harness.sh bro`.

## Done when

- `mayfly hatch` of an example task with `"harness": "bro"` reaches `done_when` on a real bro run.
- Unit tests cover argv construction (model / max-turns / cwd) and the `no_spawn` mapping.
- The smoke matrix row is filled in.

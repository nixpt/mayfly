---
name: mayfly
purpose: Short-lived single-purpose agents for Cursor, Claude, Codex, OpenCode, and friends
dcp: DCP/1.0
---

# Context

<!-- The DCP instruction layer: what an agent should *do* in this repo.
     Source of truth — adapters (CLAUDE.md / AGENTS.md / …) are generated
     from this file via `dejavue export --target <tool>`. -->

## Operating Rules

- Self-contained: no path deps on peer projects (same posture as `buckets`).
- Hatch cwd isolation via public `buckets worktree` (`--worktree <repo>`), not flame/firefly.
- Mom's kitchen: prefer `--worktree`; warn on primary source checkouts.
- Vague tasks are rejected at validate — bad task design is a mayfly bug, not a harness bug.
- Workers must not hatch other mayflies (`constraints.no_spawn`).
- Fleet Claude/Codex: prefer harness `ccf` / `cxf` (flownet env) over raw `claude` / `codex`.
- OpenCode: set `MAYFLY_OPENCODE_MODEL` (e.g. `opencode/big-pickle`) to avoid flownet default.
- Ticket IDs: `MAYFLY-NN`. Work on `agent/<name>/MAYFLY-NN` branches in worktrees.
- Never commit to `main` from an agent session — PR via task branch.

## Build / Test

```bash
cargo test
cargo build --release
mayfly validate examples/fix-test.json
mayfly validate examples/vague-ask.json   # expect reject exit 2
mayfly hatch examples/exec-true.json
MAYFLY_BIN=./target/release/mayfly ./scripts/smoke-harness.sh cursor
```

Install from release: `cargo install --git https://github.com/nixpt/mayfly --tag v0.1.2`

Foundation epoch closed 2026-08-09 — no active milestone; MAYFLY-2 is optional backlog.

## Architecture Map

```
mayfly/
├── DESIGN.md / README.md
├── schemas/task.schema.json
├── examples/              # fix-test, vague-ask, exec-true, smoke-touch-file
├── scripts/
│   ├── bump-version.sh
│   └── smoke-harness.sh
├── .dejavue/              # architectural memory
├── .jagent/               # planning board (MAYFLY-NN)
└── src/
    ├── main.rs            # CLI: validate / hatch / status / expire / list
    ├── task.rs / fuzz.rs / ttl.rs / prompt.rs / store.rs
    ├── hatch.rs / worktree.rs
    └── adapters/          # claude ccf cursor codex cxf cece opencode exec
```

## Memory

Decisions, blockers, and constraints are captured in `.dejavue/` — run
`dejavue context` for the boot packet and `dejavue recall <query>` to search.

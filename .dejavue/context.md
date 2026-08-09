---
name: mayfly
purpose: Short-lived single-purpose agents for Cursor, Claude, Codex, and friends
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
- Ticket IDs: `MAYFLY-NN`. Work on `agent/<name>/MAYFLY-NN` branches in worktrees.
- Never commit to `main` from an agent session — PR via task branch.

## Build / Test

```bash
cargo test
cargo build --release
# binary lands under $CARGO_TARGET_DIR (fleet often uses /build/release/mayfly)
mayfly validate examples/fix-test.json
mayfly validate examples/vague-ask.json   # expect reject exit 2
mayfly hatch examples/exec-true.json
```

## Architecture Map

```
mayfly/
├── DESIGN.md              # contract: hatch / aging / expire / adapters
├── schemas/task.schema.json
├── examples/              # good + vague + exec smoke tasks
├── scripts/bump-version.sh
├── .dejavue/              # architectural memory
├── .jagent/               # planning board (MAYFLY-NN)
└── src/
    ├── main.rs            # CLI: validate / hatch / status / expire / list
    ├── task.rs            # MayflyTask schema types
    ├── fuzz.rs            # vagueness gate
    ├── hatch.rs           # lifecycle + TTL watch loop
    ├── prompt.rs          # prompt envelope
    ├── store.rs           # ~/.local/state/mayfly records
    ├── ttl.rs             # 15m / 30s / 2h parser
    └── adapters/          # claude | cursor | codex | cece | exec
```

## Memory

Decisions, blockers, and constraints are captured in `.dejavue/` — run
`dejavue context` for the boot packet and `dejavue recall <query>` to search.

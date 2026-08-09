# Codebase Map

## Top-level layout

```
mayfly/
├── src/                 # CLI + hatch lifecycle + harness adapters + worktree
├── schemas/             # MayflyTask JSON Schema
├── examples/            # good / vague / exec / smoke fixtures
├── scripts/             # bump-version.sh, smoke-harness.sh
├── .dejavue/            # architectural memory (DCP/1.0)
├── .jagent/             # MAYFLY-NN planning board
├── DESIGN.md            # contract
└── README.md
```

## Key entry points

- `src/main.rs` — CLI (`validate` / `hatch` / `status` / `expire` / `list`)
- `src/hatch.rs` — provision → spawn → watch TTL/`done_when` → expire
- `src/worktree.rs` — `buckets worktree create|remove` (optional `--worktree`)
- `src/fuzz.rs` — reject vague / multi-goal / overlong TTL tasks
- `src/adapters/` — claude, ccf, cursor, codex, cxf, cece, opencode, exec

## Design invariants

- Max TTL 2h (longer → horse / `agent-launch`)
- No recursive hatch (`no_spawn`)
- Mechanical `done_when` only (no vibes)
- No peer path-deps; buckets/flame are CLI/env composition only

## External dependencies

- clap / serde / anyhow / chrono / uuid / regex / thiserror — std CLI stack
- Host harness binaries at hatch time
- Optional `buckets` on PATH for `--worktree`
- `timeout(1)` for bounded `done_when` probes on non-exec harnesses

# Codebase Map

## Top-level layout

```
mayfly/
├── src/                 # CLI + hatch lifecycle + harness adapters
├── schemas/             # MayflyTask JSON Schema
├── examples/            # good / vague / exec smoke tasks
├── scripts/             # bump-version.sh (release automation)
├── .dejavue/            # architectural memory (DCP/1.0)
├── .jagent/             # MAYFLY-NN planning board
├── DESIGN.md            # contract
└── README.md
```

## Key entry points

- `src/main.rs` — CLI (`validate` / `hatch` / `status` / `expire` / `list`)
- `src/hatch.rs` — provision → spawn → watch TTL/`done_when` → expire
- `src/fuzz.rs` — reject vague / multi-goal / overlong TTL tasks
- `src/adapters/` — claude, cursor, codex, cece, exec

## Design invariants

- Max TTL 2h (longer → horse / `agent-launch`)
- No recursive hatch (`no_spawn`)
- Mechanical `done_when` only (no vibes)
- No peer path-deps

## External dependencies

- clap / serde / anyhow / chrono / uuid / regex / thiserror — std CLI stack
- Host harness binaries (`claude`, `cursor-agent`, `codex`, …) at hatch time
- `timeout(1)` for bounded `done_when` probes on non-exec harnesses

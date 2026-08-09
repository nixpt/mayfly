# mayfly

Short-lived agents for Cursor, Claude, Codex, and friends — one task, hard TTL, then gone. Standalone binary crate (not a cargo workspace).

## Identity

- **Repository:** mayfly
- **Language:** Rust (edition 2021)
- **Binary:** `mayfly`
- **Ecosystem:** Throwaway *agents* sibling to `buckets` (throwaway *runtimes*). Below horses/`agent-launch` in the fleet stack.
- **Protocol:** CLI (`validate` / `hatch` / `status` / `expire` / `list`) + JSON task schema.

**Working this backlog?** Read `.jagent/planning/RULES.md` first — one worktree/branch per ticket + verify-before-fix.

## Crate Layout

```
mayfly/
├── Cargo.toml
├── README.md / DESIGN.md
├── schemas/task.schema.json
├── examples/
├── scripts/bump-version.sh
├── .dejavue/
├── .jagent/
└── src/
    ├── main.rs
    ├── task.rs / fuzz.rs / ttl.rs / prompt.rs / store.rs / hatch.rs
    └── adapters/{claude,cursor,codex,cece,exec}.rs
```

## Ticket prefix

`MAYFLY-NN` — see `.jagent/planning/tickets/`.

# TASKS — mayfly

See `.jagent/planning/tickets/` for full details on each `MAYFLY-N` ID.

---

## P0 — Build & Core Health ✅

- [x] `cargo test` / `cargo build --release`
- [x] validate good + reject vague
- [x] exec hatch success / fail / TTL expire
- [x] `.dejavue/`, `.jagent/`, `scripts/bump-version.sh`

---

## M1 — Isolation

- [ ] **MAYFLY-1** (M): Optional `--kitchen` integration — hatch via kitchen/worktree; refuse shared source checkout by default.

---

## M2 — Live adapters

- [ ] **MAYFLY-3** (M): Live-validate `claude` / `cursor` / `codex` adapters against real binaries with a tiny fixture task.

---

## M3 — Aging inject

- [ ] **MAYFLY-2** (M): Best-effort warn/narrow inject into live harness sessions when lifespan crosses thresholds.

---

## M4 — Release & fleet wire

- [ ] **MAYFLY-4** (S): Create remote, tag `v0.1.0`, document foreman/horse usage of mayfly.

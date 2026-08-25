# TASKS — mayfly

See `.jagent/planning/tickets/` for full details on each `MAYFLY-N` ID.

---

## M5 — Local reliability

- [ ] **MAYFLY-6** (M): Concurrent output capture, process-group expiry, and error-safe worktree cleanup. Implementation in progress on `agent/buffy/MAYFLY-6`.

---

## P0 — Build & Core Health ✅

- [x] `cargo test` / `cargo build --release`
- [x] validate good + reject vague
- [x] exec hatch success / fail / TTL expire
- [x] `.dejavue/`, `.jagent/`, `scripts/bump-version.sh`

---

## M1 — Isolation

- [x] **MAYFLY-1** (M): `--worktree` via public `buckets worktree` (create → hatch → remove --force). Flame/firefly held.

---

## M2 — Live adapters

- [x] **MAYFLY-3** (M): Live-validate `claude` / `cursor` / `codex` adapters (cursor OK; claude/codex honest auth fails).

---

## M3 — Aging inject (parked)

- [ ] **MAYFLY-2** (M): Best-effort warn/narrow inject into live harness sessions when lifespan crosses thresholds. *(post-v0.1.1 polish — not a release gate)*

---

## M4 — Release & fleet wire

- [x] **MAYFLY-4** (S): Public remote + tags + fleet usage note — **v0.1.1** released.

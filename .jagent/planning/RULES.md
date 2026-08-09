# RULES — standing discipline for anyone working this backlog

## 1. Verify before you fix

A ticket's `Backlog` status is a claim, not a fact. Before spending effort:

1. Re-run its `## Reproduction` / success criteria against current `main`.
2. If it no longer reproduces: mark `Done` with a `## Resolution` section; check the `TASKS.md` box.
3. New bugs found mid-work get their own `MAYFLY-N` ticket — don't silently fold them in.

## 2. One worktree + branch per ticket

Never work directly on `main`.

```bash
git worktree add /home/nixp/worktrees/<agent>/MAYFLY-N \
  -b agent/<agent>/MAYFLY-N origin/main
```

## 3. Commit + push at ticket boundaries

Finish → verify (`cargo test` at minimum) → commit naming the ticket → push → update planning board. Pull fresh `main` before the next ticket.

## 4. Update `.jagent/planning/` as you go

- Mark `TASKS.md` `[x]` only when verifiably done.
- Update ticket `Status` + `## Resolution`.
- New tickets use `templates/ticket.md` and the next `MAYFLY-N` number.

## Cross-references

- `.jagent/planning/TASKS.md`
- `.jagent/planning/ROADMAP.md`
- `.dejavue/` — architectural *why*

# mayfly — TODO

Repo-local agent instructions live in `.dejavue/context.md` (DCP/1.0 source of
truth). This `TODO.md` is imported by dejavue as an ambient task surface.

## Active

- [x] Scaffold CLI: validate / hatch / status / expire / list
- [x] Fuzziness gate + aging ladder + TTL expire
- [x] Adapters: claude, cursor, codex, cece (stub), exec
- [x] Rename from Meeseeks IP risk → mayfly
- [x] Wire `.dejavue/`, `.jagent/`, `scripts/bump-version.sh`
- [x] MAYFLY-1: `--worktree` via public buckets (flame/firefly held)

## Next

- MAYFLY-2: decay inject into live harness sessions (best-effort)
- MAYFLY-3: live-validate claude/codex adapters (cursor already smoked)
- MAYFLY-4: initial `v0.1.0` tag

## Reference

- `DESIGN.md` — full contract
- `.jagent/planning/` — ROADMAP / TASKS / tickets

# mayfly — TODO

Repo-local agent instructions live in `.dejavue/context.md` (DCP/1.0 source of
truth). This `TODO.md` is imported by dejavue as an ambient task surface.

## Active

- [x] Scaffold CLI: validate / hatch / status / expire / list
- [x] Fuzziness gate + aging ladder + TTL expire
- [x] Adapters: claude, cursor, codex, cece (stub), exec
- [x] Rename from Meeseeks IP risk → mayfly
- [x] Wire `.dejavue/`, `.jagent/`, `scripts/bump-version.sh`

## Next

- MAYFLY-1: kitchen/worktree integration (`--kitchen`)
- MAYFLY-2: decay inject into live harness sessions (best-effort)
- MAYFLY-3: live-validate cursor/claude/codex adapters against real binaries
- MAYFLY-4: first remote + initial `v0.1.0` tag

## Reference

- `DESIGN.md` — full contract
- `.jagent/planning/` — ROADMAP / TASKS / tickets

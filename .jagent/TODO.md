# mayfly — TODO

Repo-local agent instructions live in `.dejavue/context.md` (DCP/1.0 source of
truth). This `TODO.md` is imported by dejavue as an ambient task surface.

## Shipped (v0.1.1)

- [x] CLI: validate / hatch / status / expire / list
- [x] Fuzziness gate + aging *states* + TTL expire
- [x] Adapters: cursor, ccf, cxf, claude, codex, cece, opencode, exec
- [x] `--worktree` via public buckets
- [x] `.dejavue/` / `.jagent/` / bump-version + GitHub Release
- [x] Smoke script + harness matrix in README

## Next (optional)

- [ ] MAYFLY-2: aging/narrow *inject* into live harness sessions (best-effort)
- [ ] CHANGELOG.md if we want human release notes alongside mechanical bumps

## Reference

- `DESIGN.md` — full contract
- https://github.com/nixpt/mayfly/releases/tag/v0.1.1

# Contributing to mayfly

mayfly hatches **short-lived, single-purpose agents**: validate → hatch →
`done_when`/TTL → tear down. Its sibling posture is `buckets` — throwaway
*agents* rather than throwaway *runtimes*.

## Build and test

```sh
cargo test
cargo build --release
mayfly validate examples/fix-test.json    # end-to-end smoke
```

## Branches and tickets

- Planning board: `.jagent/planning/`, tickets are `MAYFLY-NN`.
- Work on a task branch — `agent/<name>/MAYFLY-NN`. Never commit to `main`.
- `main` carries a branch ruleset: **force-push and deletion are blocked**,
  with no bypass (admins included). Normal fast-forward merges are unaffected.
  If a push is rejected as non-fast-forward, rebase — do not reach for
  `--force`.

## Commit messages decide the version

The release job reads conventional-commit prefixes and is **gated**: only
`feat:`, `fix:`, or a breaking marker mint a version.

| prefix | effect |
|---|---|
| `feat:` | minor bump |
| `fix:` | patch bump |
| `feat!:` / `BREAKING CHANGE:` footer | major bump |
| `docs:` `chore:` `test:` `ci:` `refactor:` `style:` `perf:` `build:` | **no release** |

This gate exists because it used to be absent: `bump` defaulted to `patch` for
*every* push, so a `chore(security):` commit minted **v0.1.5**, which carries
no functional change and knocked an open PR into conflict by touching
`Cargo.toml`/`Cargo.lock`. Prefix accurately — the prefix is not cosmetic here.

Markers are matched where conventional-commits puts them: `feat:`/`fix:`/
`type!:` on the **subject line**, `BREAKING CHANGE:` as a body **footer**. A
commit body that merely *discusses* those prefixes will not trigger a release.

`BUMPVER_RELEASE_ALL=1` restores tag-on-every-push if you ever need it.

## Never commit

`.gitignore` blocks these, but the rule matters more than the mechanism —
**mayfly is a public repo**:

- `/.mcp.json` and `.jagent/*mcp*.json` — `worktree-setup` seeds a per-agent
  `.mcp.json` into every dispatched worktree carrying a **live** `joker-mcp`
  `JWT_SECRET`. Committing one publishes a live credential. This has actually
  happened twice elsewhere in the fleet.
- `/.dispatch/`, `/.claude/` — fleet-internal dispatch scaffolding, not part of
  this project.
- `.khukuri/` — audit records carrying redacted secret references and token
  hash digests.
- Generic secret shapes: `*.env`, `*.pem`, `*.key`, `id_rsa*`,
  `credentials*.json`, `.netrc`. `.env.example` is explicitly allowed.

Note the glob is `.jagent/*mcp*.json`, **not** `*.mcp.json` — the narrow form
does not match `mcp_servers.json`, which is the filename that actually leaked.
Do not narrow it.

## Adapters

Harnesses live in `src/adapters/`. An adapter targeting a *remote* agent
(rather than a local process) goes through the generalized `Harness` trait —
see `MAYFLY-5` for the Copilot cloud-agent design. Adapters touching
`src/adapters/mod.rs` conflict easily; check for open PRs on that file first.

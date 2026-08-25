# Changelog

All notable changes to mayfly are recorded here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/); versions match the `v*` git
tags. Entries under `[Unreleased]` are appended mechanically by
`scripts/bump-version.sh` when a release is cut — don't hand-edit past entries.

Backfilled 2026-08-25 from the tag history: v0.1.0–v0.1.5 shipped before this
file existed, so those entries are reconstructed from their commit ranges
rather than written at the time.

## [Unreleased]

### Fixed
- Added the missing `LICENSE-APACHE`. `Cargo.toml` and the README have both
  declared `MIT OR Apache-2.0` since the first release, but only `LICENSE-MIT`
  shipped — so GitHub detected the repo as MIT-only and anyone relying on the
  Apache-2.0 half had no text to rely on. Adding the file rather than narrowing
  the claim, since the dual grant was already published and every sibling
  project (`buckets`, `checkstand`, `crush-ast`) ships both.

## [0.1.6] - 2026-08-25

- Merge pull request #5 from nixpt/agent/buffy/MAYFLY-6
- Merge main into agent/buffy/MAYFLY-6 — resolve the v0.1.5 conflict
- fix(MAYFLY-6): harden local hatch lifecycle



## [0.1.5] - 2026-08-25

### Security
- Ignore agent/MCP artifacts and secret-shaped files. `worktree-setup` seeds a
  per-agent `.mcp.json` into every dispatched worktree carrying a live
  `joker-mcp` `JWT_SECRET`; mayfly hatches dispatched agents for a living and
  is a **public** repo, so an agent committing that file would publish a live
  credential. Nothing sensitive was ever tracked here — this is prevention.
  The glob is `.jagent/*mcp*.json`, deliberately broader than `*.mcp.json`,
  which does not match `mcp_servers.json` — the filename that actually leaked
  elsewhere in the fleet.

> **Note on this version number.** 0.1.5 was minted automatically by the
> release job on a `chore(security):` commit, because `bump` defaulted to
> "patch" for *any* push. It carries no functional change. The release job is
> now gated (see 0.1.6-dev below) so chore/docs pushes no longer mint versions.

## [0.1.4] - 2026-08-24

### Added
- `MAYFLY-5` design ticket: a GitHub Copilot cloud-agent harness adapter —
  the first adapter targeting a *remote* agent rather than a local process.

## [0.1.3] - 2026-08-23
- Foundation arc formally closed at v0.1.2 (PR #3).

## [0.1.2] - 2026-08-22
- README, planning board and dejavue refreshed for v0.1.1 (PR #2).

## [0.1.1] - 2026-08-21

### Added
- `ccf` / `cxf` / `opencode` harnesses; `MAYFLY_OPENCODE_MODEL` for
  OpenCode-native providers (PR #1).
- M2 live-adapters milestone completed.

### Fixed
- cece headless argv handling.
- Stopped tracking local cece/joker `.jagent/*.db` artifacts.

## [0.1.0] - 2026-08-09

Initial release — ephemeral single-purpose agent harness: validate → hatch →
`done_when`/TTL → tear down. `--worktree` support via the public `buckets`
crate.

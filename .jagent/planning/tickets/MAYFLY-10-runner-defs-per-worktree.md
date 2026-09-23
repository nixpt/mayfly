# MAYFLY-10: runner definitions per worktree, and a manifest edit that doesn't hit comments

**Filed:** s463 (2026-09-23), foreman, found while running `init-runners` in foreman-v9/jsess/jokersquad.

## Bugs (v0.2.0)
1. `init-runners` found the text `[commit]` inside a comment line of `.jagent/agents/.manifest`
   ("…listed under [commit]. Keep it…") and inserted the entries mid-comment. It happened in all three
   repos; their PRs were fixed by hand.
2. Runner definitions (committed, so versioned per branch) were read and written in the **main
   checkout**, even from a linked worktree. `init-runners` therefore edited source checkouts, and a
   branch-local runner change was invisible to that branch's own hatches.

## Fix
- `add_to_commit_section`: the header must be a whole line (`[commit]`), and entries are appended at
  the end of that section.
- `defs_root`: runner definitions resolve to the current worktree's top level (falling back to the main
  checkout if that worktree has no `.jagent/`). Hatch state still resolves to the main checkout
  (`project_root`).
- Tests: unit (comment containing `[commit]`, no header) + e2e (a worktree writes and reads its own
  runners, and the main checkout doesn't see them). 30/30, clippy clean.

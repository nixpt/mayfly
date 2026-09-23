# MAYFLY-8 — Runner gaps for fleet dispatch: model, read-only, budgets, success, install

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-8 |
| **Priority** | P1 |
| **Status** | Done |
| **Phase** | M5 |
| **Assignee** | foreman (nixp) |
| **Dependencies** | none (foreman-v9 FMN-4/FMN-5 depend on this) |
| **Estimated effort** | M |

## Problem

The foreman's tiered-worker plan (nixpt/foreman-v9 PR #4) sends small read jobs to mayfly on a cheap model.
Four gaps made that unsafe or impossible:
- No model knob: the claude/ccf argv had none.
- No read-only mode: claude always ran with `--dangerously-skip-permissions`, and `paths_allow`/`no_commit_to`
  were prompt text only.
- Budgets were unenforced: `budget` was parsed and never used.
- The `MAYFLY_DONE` marker alone counted as success.

The docs also said v0.1.2 / v0.1.4 while the crate is v0.1.6, and mayfly isn't installed on nixps.

## Success criteria

- [x] (B) `model`: claude/ccf `--model`, codex/cxf `-m`, opencode `-m` (beats `MAYFLY_OPENCODE_MODEL`); other harnesses refuse.
- [x] (C) `read_only`: claude/ccf run `--tools Read,Grep,Glob --allowedTools Read,Grep,Glob --permission-mode dontAsk
      --strict-mcp-config` without skip-permissions; codex/cxf run `--sandbox read-only`; other harnesses refuse at
      validate/hatch (exit 2, no hatch dir created).
- [x] (D) `budget.max_usd` → `--max-budget-usd` on claude/ccf, refused elsewhere. `budget.max_turns`: no harness
      CLI has a turn cap (not in `claude --help`), so it's documented as unenforced and `validate` prints a note.
- [x] (E) Success requires `done_when` (plus harness exit 0); the marker alone is a failed claim. `exec` compares its
      exit to `expect_exit`. A failed hatch never exits 0 (it did when the harness exited 0 but `done_when` failed;
      found by the new integration test).
- [x] (A) README/STATE versions → v0.1.6; install via `cargo install --git … --tag v0.1.6` or `--path .`; README states
      it isn't on PATH on nixps.

## Resolution

Flags verified against the installed CLIs (claude 2.1.280 `--help`, `codex exec --help`, `opencode run --help`).
18 tests (9 new unit tests covering argv construction and refusals, 2 new integration tests through the real
binary with `exec`); `cargo clippy --all-targets -D warnings` clean. That required fixing 3 lints the newer
clippy (1.98) flags in untouched code: hatch.rs term_pid, store.rs sort, worktree.rs rfind.

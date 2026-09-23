# MAYFLY-9 — Per-project runners and state in the .jagent v2 layout

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-9 |
| **Priority** | P1 |
| **Status** | Done (PR) |
| **Assignee** | foreman (nixp) |
| **Dependencies** | MAYFLY-8 (merged 7de64db) |
| **Estimated effort** | M |

## Problem
The foreman's tiered workers (foreman-v9 FMN-4) define a mayfly **read** tier per project. Captain (s463): a
project's mayfly runners install inside its `.jagent/`, in the v2 layout. Runner definitions are committed in
`.jagent/agents/mayfly/`, hatch state is gitignored in `.jagent/local/mayfly/`, and `.jagent/state/` stays
reserved for squadron. Until now mayfly had no runners, and all state went to `~/.local/state/mayfly`.

## Success criteria
- [x] State dir: `--state-dir` > `MAYFLY_STATE_DIR` > `<main checkout>/.jagent/local/mayfly` (linked worktrees →
      main checkout) > `~/.local/state/mayfly`. `list`/`status`/`expire` use the same resolution. Warns when
      `.jagent/local/` is not gitignored. No `.jagent/` means the home fallback, and nothing is created in the repo.
- [x] Runners: `.jagent/agents/mayfly/<name>.json` defaults, merged under the task via `hatch|validate --runner`
      (task wins, objects merge key by key, `task`/`done_when` rejected in runners). Unknown runner → exit 2 + list.
      `mayfly runners` lists them.
- [x] `mayfly init-runners` scaffolds `read.json` (claude, haiku, read_only, 10m, $0.25) + README. It never
      overwrites, and adds `mayfly/*.json` + `mayfly/README.md` to `.jagent/agents/.manifest [commit]` when a
      manifest exists.
- [x] Tests: 9 end-to-end (real binary, temp git repos, sandboxed HOME) + 1 unit merge test.

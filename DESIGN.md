# mayfly — Design

> Short-lived agents. One task. Then gone.

## Problem

Agent harnesses (Cursor, Claude Code, Codex, cece, …) are good at long sessions.
They are bad at **disposable single-shot work**: fix this flake, add this flag,
run this migration check. Without a contract, agents:

- expand scope ("while I'm here…")
- linger past usefulness
- spawn helpers that spawn helpers
- leave ambient state in shared checkouts

`agent-launch` + horses solve durable fleet work. mayfly solves the opposite:
**ephemeral specialists with a hard done-condition and a fixed lifespan.**

## Position in the stack

```
captain / human
      │
  foreman          ← multi-step, personas, merge waves
      │
  horse            ← scoped task, six-phase lifecycle, memory
      │
  mayfly           ← THIS — one ask, TTL, vanish
      │
  harness adapter  ← cursor | claude | codex | …
```

A mayfly is not a teammate. It is not a persona. It has no scroll, no memory
across hatches, no bridge presence. Only artifacts survive: diff, log, exit code.

## Core rules

1. **One purpose.** A hatch carries exactly one `task` and one `done_when`.
2. **Hard lifespan.** Hard `ttl` + `budget`. Past soft limit → narrow. Past hard → kill.
3. **Completes → expires.** Success tears down session/worktree. Failure reports and expires too.
4. **Vague asks are rejected.** The CLI validates before spawn. Bad task design is a mayfly bug.
5. **Workers cannot hatch others.** Only the caller (human, foreman, horse) may spawn.
   Fan-out is capped at the launcher, never delegated to a live mayfly.
6. **Harness-agnostic.** Same task schema; adapters only change argv/env.

## CLI

```
mayfly validate <task.json|->     # schema + fuzziness checks; no spawn
mayfly hatch    <task.json|->     # spawn adapter, watch TTL, expire
mayfly status   <id>              # alive | aging | narrowing | done | expired | failed
mayfly expire   <id> [--reason]   # force teardown
mayfly list                       # local hatches still on disk
```

### Hatch flow

```
validate → provision isolated cwd → write prompt → launch adapter
    → poll (done_when | ttl | budget) → collect artifacts → destroy → report
```

## Task schema (v0)

```json
{
  "task": "Fix the failing test in foo::bar::test_baz",
  "done_when": {
    "type": "command",
    "run": "cargo test -p foo bar::test_baz",
    "expect_exit": 0
  },
  "harness": "claude",
  "cwd": "/path/to/worktree",
  "ttl": "15m",
  "budget": { "max_turns": 30, "max_usd": 2.0 },
  "aging": { "warn_at": "0.5", "narrow_at": "0.75" },
  "artifacts": ["diff", "log"],
  "constraints": {
    "no_spawn": true,
    "no_commit_to": ["main", "master", "dev"],
    "paths_allow": ["src/foo/"]
  }
}
```

### `done_when` kinds

| type | meaning |
|------|---------|
| `command` | shell exits with `expect_exit` |
| `files_exist` | all listed paths exist |
| `git_diff_matches` | staged/unstaged diff touches only allowlisted paths and tests pass |
| `manual` | forbidden in v0 — mayflies don't end on vibes |

### Fuzziness gate (validate rejects)

Reject if `task` matches any of:

- no concrete noun (file, test, function, error string)
- open-ended verbs without bound: improve, polish, rethink, clean up, make better
- multiple top-level goals joined by "and also" / "then also"
- missing `done_when`
- `ttl` > 2h (not a mayfly; use a horse)

These heuristics are intentionally strict. Vague goals are the failure mode.

## Aging ladder

| life used | state | behavior |
|-----------|-------|----------|
| 0–50% | `alive` | normal work |
| 50–75% | `aging` | inject "narrow to done_when only" if harness allows |
| 75–100% | `narrowing` | revoke write outside `paths_allow`; one final push |
| 100% | `expired` | SIGTERM → SIGKILL; report `ttl_exceeded` |

A mayfly that hits TTL without `done_when` is a **task design failure**, logged as such.

## Harness adapters

Each adapter implements:

```text
spawn(prompt_path, cwd, env) -> Child
inject(child, message) -> Result<()>   // optional; best-effort for aging
collect(child) -> Artifacts
expire(child) -> ExitReport
```

| harness | spawn sketch |
|---------|----------------|
| `claude` | `claude -p … --output-format text --dangerously-skip-permissions` |
| `ccf` | same argv as `claude`; expects fleet `ccf`/flownet Anthropic env |
| `cursor` | `cursor-agent -p --yolo --trust …` |
| `codex` | `codex exec --sandbox workspace-write --skip-git-repo-check …` |
| `cxf` | `codex --profile flownet exec …` (+ `FLOWNET_TOKEN_CODEX`; fleet `cxf` shape) |
| `cece` | `cece-rs -w <cwd> -p … --afk --output-format text` |
| `opencode` | `opencode run --format json --auto --dir <cwd> …` |
| `exec` | no LLM — run `done_when.command` only (dry sanity) |

Live smoke: `./scripts/smoke-harness.sh <harness>` (see README matrix for last-verified dates).

Adapters never interpret the task. mayfly owns policy; adapters own argv.

## Prompt shape

Every hatch gets the same envelope (adapters wrap harness-specific bits):

```text
You are a mayfly: a short-lived agent with one purpose.
When that purpose is done, you stop.

PURPOSE:
<task>

DONE WHEN:
<done_when description>

CONSTRAINTS:
- Do not expand scope.
- Do not spawn other agents.
- Do not commit to main/master/dev.
- Stay inside paths_allow if set.
- Finish within your lifespan.

When DONE WHEN is satisfied, print exactly:
MAYFLY_DONE
then exit.
```

## Isolation

**v1 (shipped): `buckets worktree`** — public nixpt/buckets primitive.

```bash
mayfly hatch task.json --worktree /path/to/repo
# → buckets worktree create <repo> mayfly/<id>
# → hatch with cwd=worktree
# → buckets worktree remove … --force on done/expire/fail
```

| flag | meaning |
|------|---------|
| `--worktree <repo>` | provision cwd via buckets |
| `--branch <name>` | override default `mayfly/<id>` |
| `--from <ref>` | base for the new branch |
| `--keep-worktree` | skip teardown (debug / handoff) |

No path-dep on buckets — shells out if on PATH. Warns when `cwd` looks like a
primary source checkout (`.git` directory) and `--worktree` was not used.

**Held: flame / firefly / flare.** Those own heat/brands/fuel and zram cache roots —
a different layer. Mayflies do not wrap harnesses in `firefly spark`.

## Non-goals

- Personas, scrolls, bridge DMs
- Multi-day refactors
- Recursive mayflies (launcher may fan-out; workers may not)
- Replacing foreman / agent-launch for fleet horses

## Success metric

A good mayfly hatch:

1. validates in <100ms
2. finishes or dies within TTL
3. leaves only allowlisted artifacts
4. never touches protected branches
5. never hatches another mayfly itself

## Name & tone

Public CLI: `mayfly`. Crate: `mayfly`.
Voice: plain, finite, biological metaphor kept light (hatch / aging / expire).
No catchphrases. Status lines stay operational.

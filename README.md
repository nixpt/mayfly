# mayfly

**Short-lived agents. One task. Then gone.**

Ephemeral single-purpose workers for Cursor, Claude, Codex, OpenCode, and friends.

```bash
mayfly hatch examples/fix-test.json --worktree /path/to/repo
# → validate → buckets worktree → spawn harness → watch done_when / TTL → tear down → report
```

Not a teammate. Not a persona. A disposable specialist with a hard lifespan.

**Current release:** [v0.1.2](https://github.com/nixpt/mayfly/releases/tag/v0.1.2) · repo [nixpt/mayfly](https://github.com/nixpt/mayfly)

## Why

Long-lived agent sessions expand scope, linger, and multiply.
mayfly is the opposite contract:

| rule | meaning |
|------|---------|
| One purpose | exactly one `task` + one `done_when` |
| Hard lifespan | TTL + budget; auto-teardown |
| Vague asks die at the hatch | validate rejects open-ended tasks |
| Workers can't hatch others | no recursive spawn |
| Harness-agnostic | same schema; adapters per runner |

For durable fleet work, use horses + `agent-launch`.
For "fix this one thing and vanish", use mayfly.

## Install

```bash
cargo install --git https://github.com/nixpt/mayfly --tag v0.1.2
# or from a checkout:
cargo install --path .
# binary: mayfly
```

Needs host harness binaries on `PATH` for the runners you use (`cursor-agent`, `claude`, `codex`, …). Optional: [buckets](https://github.com/nixpt/buckets) for `--worktree`.

## Usage

```bash
# Check a task without spawning
mayfly validate examples/fix-test.json

# Hatch (dry-run prints the plan; omit --dry-run to launch)
mayfly hatch examples/fix-test.json --dry-run

# Hatch into a throwaway git worktree (public nixpt/buckets — on PATH)
mayfly hatch examples/fix-test.json --worktree /path/to/repo
mayfly hatch examples/fix-test.json --worktree . --branch mayfly/demo --keep-worktree

# Watch / force end
mayfly status <id>
mayfly expire <id>
mayfly list
```

`--worktree` shells out to `buckets worktree create/remove` (no crates.io path-dep).
Flame/firefly are intentionally not used for hatch cwd — different layer.

### Task file (minimal)

```json
{
  "task": "Fix the failing test in foo::bar::test_baz",
  "done_when": {
    "type": "command",
    "run": "cargo test -p foo bar::test_baz -- --exact",
    "expect_exit": 0
  },
  "harness": "cursor",
  "cwd": ".",
  "ttl": "15m"
}
```

See [`DESIGN.md`](DESIGN.md) for the full contract (aging ladder, adapters, fuzziness gate).

## Harnesses

| id | adapter | notes |
|----|---------|--------|
| `cursor` | `cursor-agent -p --yolo --trust` | smoke OK @ v0.1.x |
| `ccf` | same argv as `claude` | needs fleet `ccf`/flownet Anthropic env — smoke OK |
| `cxf` | `codex --profile flownet exec …` | needs `FLOWNET_TOKEN_CODEX` — smoke OK |
| `opencode` | `opencode run --auto --dir …` | set `MAYFLY_OPENCODE_MODEL` (e.g. `opencode/big-pickle`) — smoke OK |
| `claude` | `claude -p … --dangerously-skip-permissions` | needs Anthropic or `ccf` env |
| `codex` | `codex exec --sandbox workspace-write …` | raw Codex; prefer `cxf` on this fleet |
| `cece` | `cece-rs -w … -p … --afk` | fleet binary; subject to provider budget |
| `exec` | `sh -c <done_when>` | no LLM — smoke OK |

Fleet wrappers `ccf` / `cxf` are shell functions; export their env (or run under a login zsh that defines them) before `mayfly hatch` with harness `ccf`/`cxf`.

Optional smoke (needs `jq` + harness on PATH):

```bash
cargo build --release
MAYFLY_BIN=./target/release/mayfly ./scripts/smoke-harness.sh cursor
MAYFLY_BIN=./target/release/mayfly ./scripts/smoke-harness.sh cxf   # FLOWNET_TOKEN_CODEX
MAYFLY_BIN=./target/release/mayfly ./scripts/smoke-harness.sh ccf   # ANTHROPIC_* flownet env
MAYFLY_OPENCODE_MODEL=opencode/big-pickle \
  MAYFLY_BIN=./target/release/mayfly ./scripts/smoke-harness.sh opencode
```

Missing harness binaries error clearly (`harness '…' binary … not found on PATH`) instead of hanging.

## Fleet usage (horse → mayfly)

From a dispatched horse or any shell with `buckets` + `mayfly` on PATH:

```bash
mayfly hatch /path/to/task.json --worktree "$REPO"
# provisions sibling worktree via buckets, runs harness, tears down with --force
```

Prefer mechanical `done_when` + TTL ≤ 2h. Durable multi-step work stays on horses/`agent-launch`.

## Relationship to the fleet

```
foreman / horse     → multi-step, memory, merge
mayfly              → throwaway *agents* (harness + TTL)
agent-launch        → resource-controlled durable dispatch
buckets             → throwaway *runtimes* + *worktrees* (mayfly uses worktree)
flame / firefly     → workspace OS (heat/brands/fuel) — held, not wired
```

Self-contained: no path deps on peer projects.

## Agent memory & planning

| path | role |
|------|------|
| `.dejavue/` | architectural memory — `dejavue context` boot packet |
| `.jagent/` | planning board — `MAYFLY-NN` tickets in `.jagent/planning/` |
| `scripts/bump-version.sh` | conventional-commit bumps on push to `main` (see `.github/workflows/release.yml`) |

## License

MIT OR Apache-2.0

# mayfly

**Short-lived agents. One task. Then gone.**

Ephemeral single-purpose workers for Cursor, Claude, Codex, and friends.

```bash
mayfly hatch examples/fix-test.json
# → validate → spawn harness → watch done_when / TTL → tear down → report
```

Not a teammate. Not a persona. A disposable specialist with a hard lifespan.

## Why

Long-lived agent sessions expand scope, linger, and multiply.
mayfly is the opposite contract:

| rule | meaning |
|------|---------|
| One purpose | exactly one `task` + one `done_when` |
| Hard lifespan | TTL + budget; auto-teardown |
| Vague asks die at the hatch | validate rejects open-ended tasks |
| Workers can't hatch others | no recursive spawn |
| Harness-agnostic | same schema; adapters for cursor/claude/codex/… |

For durable fleet work, use horses + `agent-launch`.
For "fix this one thing and vanish", use mayfly.

## Install

```bash
cargo install --path .
# binary: mayfly
```

## Usage

```bash
# Check a task without spawning
mayfly validate examples/fix-test.json

# Hatch (dry-run prints the plan; omit --dry-run to launch)
mayfly hatch examples/fix-test.json --dry-run

# Watch / force end
mayfly status <id>
mayfly expire <id>
mayfly list
```

### Task file (minimal)

```json
{
  "task": "Fix the failing test in foo::bar::test_baz",
  "done_when": {
    "type": "command",
    "run": "cargo test -p foo bar::test_baz -- --exact",
    "expect_exit": 0
  },
  "harness": "claude",
  "cwd": ".",
  "ttl": "15m"
}
```

See [`DESIGN.md`](DESIGN.md) for the full contract (aging ladder, adapters, fuzziness gate).

## Harnesses

| id | adapter |
|----|---------|
| `claude` | Claude Code CLI |
| `cursor` | `cursor-agent` |
| `codex` | OpenAI Codex CLI |
| `cece` | fleet `cece-rs` (stub) |
| `exec` | no LLM — run `done_when` only |

## Relationship to the fleet

```
foreman / horse     → multi-step, memory, merge
mayfly              → one-shot, TTL, vanish
agent-launch        → resource-controlled durable dispatch
buckets             → throwaway *runtimes* (node/python/…)
mayfly              → throwaway *agents*
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

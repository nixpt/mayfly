# mayfly

Short-lived agents. One task. Then gone.

## Project memory

This repo uses [dejavue](https://github.com/nixpt/dejavue) for persistent architectural context.
Run `dejavue context` before making changes.
Fallback if not on PATH: `python3 .dejavue/dejavue context`

Operating rules and architecture live in `.dejavue/context.md` (DCP/1.0).
Planning board: `.jagent/planning/` (`MAYFLY-NN` tickets).

## Build / Test

```bash
cargo test
cargo build --release
mayfly validate examples/fix-test.json
mayfly hatch examples/exec-true.json
```

<!-- dejavue:discovery -->

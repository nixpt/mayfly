# MAYFLY-6 — local lifecycle reliability

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-6 |
| **Priority** | P1 |
| **Status** | In Progress |
| **Phase** | M5 |
| **Assignee** | buffy |
| **Dependencies** | MAYFLY-1, MAYFLY-3 |
| **Estimated effort** | M |

## Problem

The local hatch loop can block while draining child stdout/stderr only after
exit, allowing a noisy harness to fill a pipe before the TTL watcher can act.
Worktrees also need cleanup on every post-provision failure path, and expiring a
harness should terminate descendants rather than only the direct child.

## Success criteria

- [x] Child stdout/stderr are drained concurrently with bounded in-memory tails.
- [x] Harnesses run in their own process group on Unix and termination targets the group.
- [x] Provisioned worktrees are cleaned up after spawn, watcher, and teardown errors.
- [x] Large-output regression coverage passes without blocking and preserves output tails.
- [x] `cargo test` and `cargo build --release` pass.
- [ ] Add an integration fixture that verifies descendant termination and cleanup against a real worktree provider.

## Technical approach

- Add concurrent reader threads with a 64 KiB per-stream tail buffer.
- Put local harnesses in a Unix process group using `setpgid`, then signal the
  group on expiry, early completion, and watcher errors.
- Use a cleanup guard so worktrees are removed if planning, spawn, capture, or
  watch operations return an error.
- Keep the existing `Child` adapter boundary unchanged; remote handles remain
  the subject of MAYFLY-5.

## Files to modify

- `Cargo.toml` / `Cargo.lock` — add the Unix process-group syscall dependency.
- `src/adapters/mod.rs` — create an isolated process group for local harnesses.
- `src/hatch.rs` — concurrent output capture, termination, cleanup, and tests.

## Non-goals

- Remote execution or GitHub Copilot support (MAYFLY-5).
- Aging-message injection (MAYFLY-2).
- Full runtime enforcement of budgets and path constraints (MAYFLY-7).

## Resolution

Initial implementation is on branch `agent/buffy/MAYFLY-6`. The remaining
integration fixture should land before marking the ticket Done.

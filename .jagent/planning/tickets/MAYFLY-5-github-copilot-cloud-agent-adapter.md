# MAYFLY-5 — GitHub Copilot cloud-agent harness adapter

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-5 |
| **Priority** | P3 |
| **Status** | Backlog |
| **Phase** | design |
| **Assignee** | unassigned |
| **Dependencies** | none |
| **Estimated effort** | M |

## Problem

Captain's idea (2026-08-14, foreman s433): mayfly is harness-agnostic by
design (`spawn`/`inject`/`collect`/`expire` per adapter, adapters own argv,
mayfly owns policy — see `DESIGN.md` "Harness adapters"). GitHub now ships
**custom cloud agents** (`.github/agents/<NAME>.md` — repo/org/enterprise
scoped, Markdown + YAML frontmatter, assignable to issues/PRs, docs:
https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-custom-agents).
There is currently no adapter that lets mayfly hatch a Copilot cloud agent as
a worker, and no way to apply mayfly's TTL/`done_when`/fuzziness-gate
discipline to a Copilot-cloud-agent task.

Concrete motivating use case: **watch CI after a merge, and if it goes red,
hand the fix to a bounded, disposable worker** — today that's either a bare
Copilot-agent assignment (open-ended, no TTL, no `done_when` completion
check) or a full horse/foreman dispatch (too heavy for a single CI-fix
task). Neither matches mayfly's actual niche, which is exactly this shape.

## The real architectural gap (not just "write an adapter")

Every existing adapter (`cursor`, `claude`/`ccf`, `codex`/`cxf`, `opencode`,
`cece`, `exec`) implicitly owns a **local `Child` process** —
`spawn(prompt_path, cwd, env) -> Child` is the trait today. A Copilot cloud
agent is not a local process; it's a task assigned via GitHub's API
(`gh`/REST) that runs in GitHub's own sandbox, with no local PID to hold,
signal, or wait on. The adapter contract's four ops still map conceptually:

- `spawn` → API call assigning the custom agent to an issue/PR with the
  task prompt as the issue/PR body or comment
- `inject` → a PR/issue comment (best-effort — same as today's aging-ladder
  inject, MAYFLY-2, parked)
- `collect` → pull the resulting diff/commits/check-run log via `gh`
- `expire` → unassign / close the issue-PR, same TTL-driven teardown
  mayfly already does for local processes

But this needs a real decision, not just new adapter code: does the
`Harness` trait get a second implementation shape (a "remote handle"
variant alongside `Child`), or does the whole trait get generalized to an
abstract handle from the start? Answering that is most of this ticket's
actual design work — the CI-polling logic itself is comparatively simple.

## Success criteria (design-stage — this ticket is scoping, not landing code)

- [ ] Decision made and written up: how the `Harness`/adapter trait
      generalizes (or doesn't) to support a non-`Child` remote handle
- [ ] A concrete task-schema example for the "watch CI, fix if red" case —
      what `done_when` looks like against a GitHub Actions check-run
      status (polling shape, not a blocking command)
- [ ] TTL/expire semantics defined for a remote agent that mayfly can't
      kill directly (unassign vs. close vs. just stop polling and report)
- [ ] Auth/scoping question answered: does this run with the fleet's
      existing `gh` auth, or does it need its own token/app installation
      for whichever org-level `.github`/`.github-private` repo hosts the
      custom agent
- [ ] Explicit non-goal check against `DESIGN.md`'s existing Non-goals
      list (no recursive mayflies, no replacing foreman/horses) — confirm
      this adapter doesn't quietly turn into a second orchestration layer

## Notes

Purely a design ticket at filing time — no code, no adapter implementation
yet. Came out of captain + foreman conversation in foreman session 433
alongside separate research into GitHub's custom-cloud-agents docs (see
workspace memory: `github-copilot-custom-cloud-agents`). Revisit
`DESIGN.md`'s "Harness adapters" and "Non-goals" sections before starting
implementation.

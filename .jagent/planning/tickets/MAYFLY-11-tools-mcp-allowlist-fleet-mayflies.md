# MAYFLY-11 — tools + mcp_servers allowlist for claude; fleet mayflies

| Field | Value |
|-------|-------|
| **ID** | MAYFLY-11 |
| **Priority** | P1 |
| **Status** | Done |
| **Phase** | M5 |
| **Assignee** | cursor-mf11 |
| **Dependencies** | MAYFLY-8 |
| **Estimated effort** | M |

## Problem

Fleet mayflies (mailer on Haiku, inbox digest, PR survey) need a curated Claude tool + MCP
allowlist. Today `read_only` pins Read/Grep/Glob with no MCP, and the default path uses
`--dangerously-skip-permissions`. A mailer must reach only mailgate read/draft tools — never
skip-permissions, shell, or `mail_send`.

## Success criteria

- [x] `tools` / `mcp_servers` on the task schema; claude/ccf argv never skip-permissions when `tools` is set
- [x] validate rejects tools+read_only, mcp_servers without tools, mcp tool whose server is missing, wrong harness
- [x] hatch-local mcp-config is 0600 and removed on dry-run / hatch end / expire
- [x] `examples/fleet/{mailer,inbox-digest,pr-survey}.json` validate; README documents them
- [x] tests, clippy `-D warnings`, fmt clean; PR open

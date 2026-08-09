# Decisions


## 2026-08-09T00:02:10-05:00 — Name the project mayfly, not Meeseeks

Reason:
Mr. Meeseeks name/catchphrases are Rick and Morty IP; ephemeral single-purpose agent pattern is not. mayfly keeps the metaphor (short life, one act) without trademark risk.

Rejected alternatives:
- **keep Meeseeks branding with disclaimer**: still distinctive IP expression
- **generic names like oneshot/tempagent**: weaker product identity next to buckets


## 2026-08-09T00:02:11-05:00 — Self-contained crate with harness adapters, no peer path-deps

Reason:
Same posture as buckets — throwaway agent tool any runner can call without pulling the fleet DAG.

Rejected alternatives:
- **path-dep on agent-launch/cece-rs**: couples mayfly to squadron layout and slows portability


## 2026-08-09T00:02:11-05:00 — Skip done_when re-probe for exec harness; timeout probes for others

Reason:
Exec child IS the done_when command; re-running sleep/hang probes blocked the TTL loop (caught in TTL smoke test).

Rejected alternatives:
- **always poll done_when unboundedly**: TTL starvation
- **no mid-flight done_when checks for LLM harnesses**: loses early-cease


## 2026-08-09T00:14:26-05:00 — Use public buckets worktree for hatch isolation; hold flame/firefly

Reason:
buckets is published and already the fleet throwaway-workspace primitive (kitchen sits on it). Flame/firefly own heat/brands/fuel — a different layer; wrapping harnesses in firefly spark would duplicate and fight host auth/network needs.

Rejected alternatives:
- **kitchen enter/clean**: horse ship/PR lifecycle; refuse-dirty fights mayfly --force teardown
- **firefly spark as hatch sandbox**: wrong product boundary; mayfly is the agent loop outside the room
- **reimplement git worktree inside mayfly**: would fork buckets


## 2026-08-09T00:22:33-05:00 — Align harness argv with live CLIs; smoke fails honestly on auth

Reason:
codex --full-auto is deprecated (use --sandbox workspace-write); claude headless needs --dangerously-skip-permissions. Live smoke proved cursor OK; claude/codex launch but need valid credentials — mayfly should surface that, not hang.

Rejected alternatives:
- **skip live smoke until all auths green**: hides argv drift
- **keep --full-auto forever**: warns and may vanish


## 2026-08-09T00:38:04-05:00 — Prefer ccf/cxf harness ids over raw claude/codex on this fleet

Reason:
ccf/cxf are zsh wrappers that inject flownet Anthropic/Codex env; raw claude/codex fail honestly on missing auth. Adapters share argv with claude/codex but expect that env — document and smoke those ids as the fleet path.

Rejected alternatives:
- **require users to source ccf before hatch with harness=claude**: opaque and easy to miss
- **bake flownet secrets into mayfly**: wrong boundary


## 2026-08-09T00:38:05-05:00 — OpenCode model via MAYFLY_OPENCODE_MODEL env

Reason:
opencode defaults can route through flownet; setting MAYFLY_OPENCODE_MODEL (e.g. opencode/big-pickle) keeps smoke and local hatches on OpenCode-native providers without hardcoding a model in the task schema.

Rejected alternatives:
- **hardcode model in adapter**: fights per-user provider choice
- **require model in every task JSON**: noisy for the common case


## 2026-08-09T00:38:05-05:00 — Ship v0.1.1 with aging states but park inject (MAYFLY-2)

Reason:
TTL aging ladder states already flip; per-harness inject is best-effort polish and not needed for the public release gate. Keeps the crate small and honest about what is verified.

Rejected alternatives:
- **block release on aging inject**: delays useful hatch loop


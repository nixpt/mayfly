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


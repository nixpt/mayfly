#!/usr/bin/env bash
# Optional live smoke: hatch examples/smoke-touch-file.json under one harness.
#
#   ./scripts/smoke-harness.sh cursor
#   MAYFLY_SMOKE_HARNESS=claude ./scripts/smoke-harness.sh
#
# Requires: mayfly on PATH (or set MAYFLY_BIN), jq, and the harness binary.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HARNESS="${1:-${MAYFLY_SMOKE_HARNESS:-}}"
MAYFLY_BIN="${MAYFLY_BIN:-mayfly}"
STATE="${MAYFLY_STATE_DIR:-$(mktemp -d /tmp/mayfly-smoke-XXXXXX)}"

[[ -n "$HARNESS" ]] || {
  echo "usage: $0 <claude|ccf|cursor|codex|cxf|cece|opencode|exec>" >&2
  exit 2
}

command -v jq >/dev/null || { echo "smoke: jq required" >&2; exit 1; }
command -v "$MAYFLY_BIN" >/dev/null || { echo "smoke: $MAYFLY_BIN not on PATH" >&2; exit 1; }

PROOF="$ROOT/examples/smoke-proof.txt"
rm -f "$PROOF"

TASK="$(mktemp)"
jq --arg h "$HARNESS" '.harness = $h | .cwd = "."' \
  "$ROOT/examples/smoke-touch-file.json" >"$TASK"

echo "smoke: harness=$HARNESS state=$STATE"
set +e
"$MAYFLY_BIN" --state-dir "$STATE" hatch "$TASK"
rc=$?
set -e
rm -f "$TASK"

if [[ $rc -eq 0 ]] && grep -qx MAYFLY_SMOKE_OK "$PROOF" 2>/dev/null; then
  echo "smoke: OK ($HARNESS)"
  rm -f "$PROOF"
  exit 0
fi

echo "smoke: FAIL ($HARNESS) exit=$rc" >&2
[[ -f "$PROOF" ]] && echo "smoke: proof was: $(cat "$PROOF")" >&2
exit "${rc:-1}"

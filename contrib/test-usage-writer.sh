#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
#
# Tests for usage-writer.sh. Runs it against crafted render JSON in a temp dir
# (via CLAUDE_USAGE_HISTORY) and asserts on the persisted lines. No real files
# are touched. Run: bash contrib/test-usage-writer.sh
set -u
here=$(cd "$(dirname "$0")" && pwd)
writer="$here/usage-writer.sh"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
export CLAUDE_USAGE_HISTORY="$tmp/usage-history.jsonl"
export CLAUDE_USAGE_HEARTBEAT=120

fail=0
ok()   { printf '  ok   %s\n' "$1"; }
bad()  { printf '  FAIL %s\n' "$1"; fail=1; }
lines() { [ -f "$CLAUDE_USAGE_HISTORY" ] && wc -l < "$CLAUDE_USAGE_HISTORY" || echo 0; }
run()  { printf '%s' "$1" | bash "$writer" >/dev/null 2>&1; }

s_unix='{"rate_limits":{"five_hour":{"used_percentage":45,"resets_at":1782939600},"seven_day":{"used_percentage":80,"resets_at":1783022400}}}'
s_chg='{"rate_limits":{"five_hour":{"used_percentage":46,"resets_at":1782939600},"seven_day":{"used_percentage":80,"resets_at":1783022400}}}'
s_iso='{"rate_limits":{"five_hour":{"used_percentage":12.5,"resets_at":"2026-07-01T22:00:00Z"},"seven_day":{"used_percentage":8,"resets_at":"2026-07-08T00:00:00Z"}}}'
s_none='{"model":{"display_name":"Opus"}}'

# 1. first write + exact format (fractions, unix resets, %g-clean numbers)
run "$s_unix"
got=$(sed 's/"ts": [0-9]*/"ts": T/' "$CLAUDE_USAGE_HISTORY")
want='{"ts": T, "session": 0.45, "weekly": 0.8, "session_reset": 1782939600, "weekly_reset": 1783022400}'
[ "$got" = "$want" ] && ok "writes applet-format line" || bad "line format: got [$got]"

# 2. valid JSON the applet's serde can read
tail -1 "$CLAUDE_USAGE_HISTORY" | jq -e '.session and .weekly and .session_reset' >/dev/null 2>&1 \
  && ok "line is valid JSON with required keys" || bad "line not valid JSON"

# 3. dedup: identical sample does not append
run "$s_unix"; [ "$(lines)" -eq 1 ] && ok "dedup skips identical sample" || bad "dedup: $(lines) lines, want 1"

# 4. change appends
run "$s_chg"; [ "$(lines)" -eq 2 ] && ok "changed sample appends" || bad "change: $(lines) lines, want 2"

# 5. no rate_limits -> no write, no crash
run "$s_none"; [ "$(lines)" -eq 2 ] && ok "no rate_limits leaves history untouched" || bad "no-rl: $(lines) lines, want 2"

# 6. ISO-8601 resets_at normalised to unix seconds
run "$s_iso"
iso_reset=$(tail -1 "$CLAUDE_USAGE_HISTORY" | jq -r '.session_reset')
exp_reset=$(date -d "2026-07-01T22:00:00Z" +%s)
[ "$iso_reset" = "$exp_reset" ] && ok "ISO resets_at normalised to unix" || bad "iso reset: got $iso_reset want $exp_reset"

# 7. heartbeat: unchanged sample re-writes once the last line ages past 120s
last=$(tail -1 "$CLAUDE_USAGE_HISTORY"); before=$(lines)
python3 - "$CLAUDE_USAGE_HISTORY" <<'PY'
import sys, json, time
p = sys.argv[1]; ls = open(p).read().splitlines()
d = json.loads(ls[-1]); d["ts"] = int(time.time()) - 200; ls[-1] = json.dumps(d)
open(p, "w").write("\n".join(ls) + "\n")
PY
run "$s_iso"; [ "$(lines)" -eq $((before + 1)) ] && ok "heartbeat re-writes stale-but-unchanged sample" || bad "heartbeat: $(lines) lines, want $((before + 1))"

echo
[ "$fail" -eq 0 ] && echo "PASS" || echo "FAIL"
exit "$fail"

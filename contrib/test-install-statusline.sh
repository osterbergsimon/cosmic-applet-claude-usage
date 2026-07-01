#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
#
# Tests for install-statusline.sh, against a temp CLAUDE_CONFIG_DIR. No real
# config is touched. Run: bash contrib/test-install-statusline.sh
set -u
here=$(cd "$(dirname "$0")" && pwd)
installer="$here/install-statusline.sh"

fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1"; fail=1; }

fresh_dir() { local d; d=$(mktemp -d); echo "$d"; }
sl_cmd() { jq -r '.statusLine.command // empty' "$1/settings.json"; }

# 1. fresh install: no settings.json -> writer installed + statusLine set
d=$(fresh_dir)
CLAUDE_CONFIG_DIR="$d" bash "$installer" >/dev/null 2>&1
[ -x "$d/usage-writer.sh" ] && ok "installs writer (executable)" || bad "writer not installed"
[ "$(sl_cmd "$d")" = "bash $d/usage-writer.sh" ] && ok "sets statusLine when none exists" || bad "statusLine not set: [$(sl_cmd "$d")]"
rm -rf "$d"

# 2. existing unrelated statusLine -> preserved, exits 0
d=$(fresh_dir)
echo '{"statusLine":{"type":"command","command":"bash /my/prompt.sh"}}' > "$d/settings.json"
CLAUDE_CONFIG_DIR="$d" bash "$installer" >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" = 0 ] && ok "exits 0 with an existing statusLine" || bad "nonzero exit ($rc) with existing statusLine"
[ "$(sl_cmd "$d")" = "bash /my/prompt.sh" ] && ok "does not clobber an existing statusLine" || bad "clobbered existing statusLine: [$(sl_cmd "$d")]"
[ -x "$d/usage-writer.sh" ] && ok "still installs writer alongside existing statusLine" || bad "writer not installed in existing-statusLine case"
rm -rf "$d"

# 3. idempotent: re-running after it set the statusLine changes nothing
d=$(fresh_dir)
CLAUDE_CONFIG_DIR="$d" bash "$installer" >/dev/null 2>&1
first=$(sl_cmd "$d")
CLAUDE_CONFIG_DIR="$d" bash "$installer" >/dev/null 2>&1
[ "$(sl_cmd "$d")" = "$first" ] && ok "idempotent on re-run" || bad "statusLine changed on re-run"
# exactly one backup from the single mutation
bak=$(find "$d" -name 'settings.json.bak.*' | wc -l)
[ "$bak" -eq 1 ] && ok "backs up settings.json once (no backup on no-op re-run)" || bad "unexpected backup count: $bak"
rm -rf "$d"

# 4. preserves other settings keys when injecting statusLine
d=$(fresh_dir)
echo '{"tui":"fullscreen","env":{}}' > "$d/settings.json"
CLAUDE_CONFIG_DIR="$d" bash "$installer" >/dev/null 2>&1
[ "$(jq -r '.tui' "$d/settings.json")" = "fullscreen" ] && ok "preserves unrelated settings keys" || bad "dropped unrelated settings keys"
rm -rf "$d"

# 5. --check: not wired when nothing is configured (exit 1)
d=$(fresh_dir)
CLAUDE_CONFIG_DIR="$d" bash "$installer" --check >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" = 1 ] && ok "--check exits 1 when not wired" || bad "--check exit was $rc, want 1"
rm -rf "$d"

# 6. --check: wired after install (exit 0)
d=$(fresh_dir)
CLAUDE_CONFIG_DIR="$d" bash "$installer" >/dev/null 2>&1
CLAUDE_CONFIG_DIR="$d" bash "$installer" --check >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" = 0 ] && ok "--check exits 0 after install" || bad "--check exit was $rc, want 0"
rm -rf "$d"

# 7. --check: detects the pasted persist block in a user's own statusLine script
d=$(fresh_dir)
printf '#!/usr/bin/env bash\n# writes usage-history.jsonl here\n' > "$d/my-statusline.sh"
echo "{\"statusLine\":{\"type\":\"command\",\"command\":\"bash $d/my-statusline.sh\"}}" > "$d/settings.json"
CLAUDE_CONFIG_DIR="$d" bash "$installer" --check >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" = 0 ] && ok "--check detects pasted block in own script" || bad "--check missed pasted block (exit $rc)"
# and install mode leaves that setup untouched
CLAUDE_CONFIG_DIR="$d" bash "$installer" >/dev/null 2>&1
[ "$(sl_cmd "$d")" = "bash $d/my-statusline.sh" ] && ok "install leaves a block-fed statusLine untouched" || bad "clobbered block-fed statusLine"
rm -rf "$d"

# 8. --check: unrelated statusLine is reported not wired (exit 1)
d=$(fresh_dir)
echo '{"statusLine":{"type":"command","command":"bash /my/prompt.sh"}}' > "$d/settings.json"
CLAUDE_CONFIG_DIR="$d" bash "$installer" --check >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" = 1 ] && ok "--check reports unrelated statusLine as not wired" || bad "--check false-positive (exit $rc)"
rm -rf "$d"

# 9. --check with FRESH history + wired statusLine -> exit 0, reports wired
#    (regression: the freshness reporter must not trip `set -e` on fresh data)
d=$(fresh_dir)
CLAUDE_CONFIG_DIR="$d" bash "$installer" >/dev/null 2>&1
printf '{"ts": %s, "session": 0.4, "weekly": 0.5, "session_reset": 0, "weekly_reset": 0}\n' "$(date +%s)" > "$d/usage-history.jsonl"
out=$(CLAUDE_CONFIG_DIR="$d" bash "$installer" --check 2>&1); rc=$?
{ [ "$rc" = 0 ] && printf '%s' "$out" | grep -q 'wired: *yes'; } \
  && ok "--check exits 0 and reports wired with fresh history" || bad "fresh-history regression (exit $rc): $out"
rm -rf "$d"

echo
[ "$fail" -eq 0 ] && echo "PASS" || echo "FAIL"
exit "$fail"

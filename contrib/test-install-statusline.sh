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

echo
[ "$fail" -eq 0 ] && echo "PASS" || echo "FAIL"
exit "$fail"

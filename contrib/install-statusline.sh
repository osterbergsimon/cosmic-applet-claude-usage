#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
#
# install-statusline.sh — install usage-writer.sh and wire it into Claude Code
# so the applet gets fed. Safe to re-run.
#
#   (default)  Copy usage-writer.sh to ~/.claude/usage-writer.sh (a stable,
#              writable path, independent of any read-only /nix/store or /usr
#              install). If no statusLine is configured, point Claude Code's
#              statusLine at it (backing up settings.json first). An existing
#              statusLine is left untouched — instructions are printed instead.
#
#   --check    Report whether the applet is currently wired to be fed, and how
#              fresh the data is. Changes nothing. Exit 0 if wired, else 1.
#
#   --help     This message.
#
# Note: nothing that touches the statusLine survives you regenerating it (e.g.
# `/statusline`), since that overwrites settings.json's statusLine command. The
# most durable setup is to paste the "persist" block from usage-writer.sh into
# your own statusLine script — it travels with your edits. After any statusLine
# change, `--check` tells you in one line whether feeding still works.
#
# Honours CLAUDE_CONFIG_DIR (default ~/.claude), matching Claude Code itself.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
src="$here/usage-writer.sh"
claude_dir="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
dest="$claude_dir/usage-writer.sh"
settings="$claude_dir/settings.json"
history="${CLAUDE_USAGE_HISTORY:-$claude_dir/usage-history.jsonl}"
cmd="bash $dest"

command -v jq >/dev/null 2>&1 || { echo "error: jq is required" >&2; exit 1; }

# True if the statusLine command feeds the applet: either it runs usage-writer.sh
# directly, or it runs a script that contains the persist block (writes the
# history file). Detecting the pasted-block case is why we grep referenced files.
is_wired() {
  local command="$1" tok exp
  [ -n "$command" ] || return 1
  printf '%s' "$command" | grep -q 'usage-writer.sh' && return 0
  for tok in $command; do
    exp="${tok/#\~/$HOME}"
    [ -f "$exp" ] && grep -q 'usage-history.jsonl' "$exp" 2>/dev/null && return 0
  done
  return 1
}

report_freshness() {
  if [ -f "$history" ]; then
    local last_ts age
    last_ts=$(tail -1 "$history" 2>/dev/null | jq -r '.ts // 0' 2>/dev/null | cut -d. -f1)
    age=$(( $(date +%s) - ${last_ts:-0} ))
    echo "data:       last sample ${age}s ago ($history)"
    if [ "$age" -gt 900 ]; then
      echo "            (stale — feeding only runs while Claude Code is open)"
    fi
  else
    echo "data:       no history file yet ($history)"
  fi
}

# ---- --check ----------------------------------------------------------------
if [ "${1:-}" = "--check" ]; then
  current=""
  [ -f "$settings" ] && current=$(jq -r '.statusLine.command // empty' "$settings")
  echo "statusLine: ${current:-<none configured>}"
  report_freshness
  if is_wired "$current"; then
    echo "wired:      yes — the statusLine feeds the applet."
    exit 0
  fi
  echo "wired:      NO — the applet is not being fed."
  echo "            run '$here/install-statusline.sh' (or paste the persist"
  echo "            block from usage-writer.sh into your statusLine script)."
  exit 1
fi

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  sed -n '3,25p' "$0" | sed 's/^# \{0,1\}//'
  exit 0
fi

# ---- install ----------------------------------------------------------------
[ -f "$src" ] || { echo "error: $src not found" >&2; exit 1; }

mkdir -p "$claude_dir"
install -m 0755 "$src" "$dest"
echo "installed writer → $dest"

[ -f "$settings" ] || echo '{}' > "$settings"
current=$(jq -r '.statusLine.command // empty' "$settings")

if [ -z "$current" ]; then
  cp "$settings" "$settings.bak.$(date +%s)"
  tmp=$(mktemp)
  jq --arg cmd "$cmd" '.statusLine = {type: "command", command: $cmd}' "$settings" > "$tmp"
  mv "$tmp" "$settings"
  echo "set statusLine → $cmd"
  echo "(backup saved next to settings.json)"
elif is_wired "$current"; then
  echo "statusLine already feeds the applet — nothing to change."
else
  cat <<EOF

You already have a statusLine, left untouched:
    $current

To feed the applet too, the most durable option is to paste the "persist" block
from usage-writer.sh directly into your own statusLine script — it stays put
when you edit your prompt. Alternatively, tee the render JSON through the writer
from a small wrapper:

    #!/usr/bin/env bash
    input=\$(cat)
    printf '%s' "\$input" | $cmd >/dev/null
    printf '%s' "\$input" | <your-existing-statusline>

Verify feeding with:  $here/install-statusline.sh --check
EOF
fi

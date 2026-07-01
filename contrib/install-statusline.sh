#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
#
# install-statusline.sh — install usage-writer.sh and wire it into Claude Code
# so the applet gets fed. Safe to re-run.
#
#   - Copies usage-writer.sh to ~/.claude/usage-writer.sh (a stable, writable
#     path, independent of any read-only /nix/store or /usr install).
#   - If no statusLine is configured, points Claude Code's statusLine at it
#     (backing up settings.json first).
#   - If you already have a statusLine, it is left untouched and instructions
#     are printed for teeing the render JSON through the writer.
#
# Honours CLAUDE_CONFIG_DIR (default ~/.claude), matching Claude Code itself.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
src="$here/usage-writer.sh"
claude_dir="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
dest="$claude_dir/usage-writer.sh"
settings="$claude_dir/settings.json"
cmd="bash $dest"

command -v jq >/dev/null 2>&1 || { echo "error: jq is required" >&2; exit 1; }
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
elif printf '%s' "$current" | grep -q 'usage-writer.sh'; then
  echo "statusLine already runs usage-writer.sh — nothing to change."
else
  cat <<EOF

You already have a statusLine, left untouched:
    $current

To feed the applet too, tee the render JSON through the writer. Point your
statusLine at a small wrapper that runs both:

    #!/usr/bin/env bash
    input=\$(cat)
    printf '%s' "\$input" | $cmd >/dev/null
    printf '%s' "\$input" | <your-existing-statusline>

Alternatively, paste the "persist" block from usage-writer.sh into your own
script. See the project README, "Feeding the applet".
EOF
fi

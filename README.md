# cosmic-applet-claude-usage

A minimal COSMIC panel applet showing Claude Code usage as a quiet color-coded
indicator. Green → amber → red as you approach your session (5h) or weekly (7d)
limit. Hover for exact percentages; click for reset countdowns.

## Data source

Reads the last line of `~/.claude/usage-history.jsonl`, which Claude Code's
status line appends to while running. Values are "last known" between sessions;
the indicator dims when data is older than `stale_after`.

## Build & install (NixOS)

Rust is not installed globally; the toolchain and libcosmic's native build/runtime
dependencies come from the project `flake.nix` devShell. All `cargo`/`just`
commands run inside `nix develop -c …`. The first build is slow — it fetches and
compiles libcosmic from source.

Dev iteration (no sudo, no /usr):

    nix develop -c just install-dev   # → ~/.local/bin + ~/.local/share/applications

Permanent install via home-manager — import `nix/overlay.nix` (in this repo) into
your dotfiles flake's `nixpkgs.overlays`, add `cosmic-applet-claude-usage` to your
home packages, fill in the libcosmic `outputHashes` (first build prints the hash),
then:

    sudo nixos-rebuild switch --flake ~/nixos-config#HOSTNAME

Then add `co.osterberg.ClaudeUsage` to the panel via COSMIC Settings → Panel → Applets.

> Packaging is versioned in this repo at `nix/overlay.nix` — an overlay you wire
> into your dotfiles flake. See the comment at the top of that file.

## Config

Stored via cosmic-config (`co.osterberg.ClaudeUsage` v1). Defaults are used on
first run and whenever a key is unset or unreadable. Keys:

| Key            | Values                                   | Default     |
|----------------|------------------------------------------|-------------|
| scope          | session, weekly, worst, both             | worst       |
| style          | color-dot, fill-bar, fill-color, ring, ring-color | color-dot |
| show_percent   | true, false                              | false       |
| reset_display  | none, text, compact, glow, dual-ring, track | none     |
| thresholds     | { amber, red } fractions                 | 0.50 / 0.80 |
| stale_after    | seconds                                  | 600         |
| history_path   | path override (optional)                 | (unset)     |

## Layout

- `flake.nix` — Nix devShell providing the Rust toolchain + Wayland/Vulkan libs
- `nix/overlay.nix` — nixpkgs overlay packaging the applet (wire into your dotfiles)
- `Cargo.toml` — crate manifest; libcosmic pinned by `rev`
- `src/main.rs` — the libcosmic applet
- `src/config.rs` — config struct + cosmic-config loader
- `data/co.osterberg.ClaudeUsage.desktop` — COSMIC applet desktop entry
- `justfile` — build / dev-install / test recipes

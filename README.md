# cosmic-applet-claude-usage

A minimal [COSMIC](https://github.com/pop-os/cosmic-epoch) top-panel applet that
shows Claude usage as a colored dot.

## Status

Early scaffolding. Currently renders a static placeholder indicator in the panel.

## Building (NixOS)

Rust is not installed globally; the toolchain and libcosmic's native build/runtime
dependencies come from the project `flake.nix` devShell. All `cargo`/`just`
commands run inside `nix develop -c …`.

```bash
nix develop -c cargo build --release
```

The first build is slow — it fetches and compiles libcosmic from source.

## Dev install (no sudo, no /usr)

```bash
nix develop -c just install-dev
```

This drops the binary in `~/.local/bin` and the desktop entry in
`~/.local/share/applications`, then add `co.osterberg.ClaudeUsage` to the panel
via COSMIC Settings → Desktop → Panel → Applets.

## Layout

- `flake.nix` — Nix devShell providing the Rust toolchain + Wayland/Vulkan libs
- `Cargo.toml` — crate manifest; libcosmic pinned by `rev`
- `src/main.rs` — the libcosmic applet
- `data/co.osterberg.ClaudeUsage.desktop` — COSMIC applet desktop entry
- `justfile` — build / dev-install / test recipes

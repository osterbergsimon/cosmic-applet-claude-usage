bindir := env_var('HOME') / '.local/bin'
libexecdir := env_var('HOME') / '.local/libexec'
bin-real := libexecdir / 'cosmic-applet-claude-usage-bin'
bin-wrapper := bindir / 'cosmic-applet-claude-usage'
desktop-dst := env_var('HOME') / '.local/share/applications/co.osterberg.ClaudeUsage.desktop'

build:
    cargo build --release

# Dev install (no sudo, no /usr) — for iterating before the Nix overlay is wired in.
# winit dlopens libwayland/libGL/vulkan at runtime, and the panel launches the
# binary WITHOUT the devShell env — so bake the devShell's LD_LIBRARY_PATH (set
# while this runs under `nix develop`) into a wrapper. NOTE: those /nix/store
# paths are not GC-roots, so this dev install breaks on `nix-collect-garbage`;
# the Nix overlay is the durable install.
install-dev: build
    install -Dm0755 target/release/cosmic-applet-claude-usage {{bin-real}}
    mkdir -p {{bindir}}
    printf '#!/bin/sh\nexport LD_LIBRARY_PATH=%s\nexec %s "$@"\n' "$LD_LIBRARY_PATH" "{{bin-real}}" > {{bin-wrapper}}
    chmod +x {{bin-wrapper}}
    mkdir -p $(dirname {{desktop-dst}})
    sed 's|^Exec=.*|Exec={{bin-wrapper}}|' data/co.osterberg.ClaudeUsage.desktop > {{desktop-dst}}

test:
    cargo test

bin-dst := env_var('HOME') / '.local/bin/cosmic-applet-claude-usage'
desktop-dst := env_var('HOME') / '.local/share/applications/co.osterberg.ClaudeUsage.desktop'

build:
    cargo build --release

# Dev install (no sudo, no /usr) — for iterating before the Nix derivation exists.
install-dev: build
    install -Dm0755 target/release/cosmic-applet-claude-usage {{bin-dst}}
    mkdir -p $(dirname {{desktop-dst}})
    sed 's|^Exec=.*|Exec={{bin-dst}}|' data/co.osterberg.ClaudeUsage.desktop > {{desktop-dst}}

test:
    cargo test

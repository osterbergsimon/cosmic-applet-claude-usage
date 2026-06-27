# nixpkgs overlay packaging this applet.
#
# This file is versioned alongside the source so packaging stays in sync with
# the code. To install permanently, wire it into your dotfiles flake later:
#   1. copy/import this overlay into ~/nixos-config (e.g. overlays/claude-usage),
#      adjusting `src` to point at the checkout you build from;
#   2. add `(import ./overlays/claude-usage)` to `nixpkgs.overlays`;
#   3. add `cosmic-applet-claude-usage` to your home.packages;
#   4. run `nix build` once to learn the libcosmic git-dependency hash, copy the
#      `got: sha256-…` value into `outputHashes` below, then `nixos-rebuild switch`.
#
# Paths here are relative to this file's location (nix/), so `src = ../.;` is the
# repo root and `cargoLock.lockFile = ../Cargo.lock;`.

final: prev:

{
  cosmic-applet-claude-usage = final.rustPlatform.buildRustPackage rec {
    pname = "cosmic-applet-claude-usage";
    version = "0.1.0";

    src = ../.;
    cargoLock = {
      lockFile = ../Cargo.lock;
      # libcosmic is a git dependency; pin its hash. Run the build once and
      # copy the "got:" hash nix prints into outputHashes below.
      outputHashes = {
        # "libcosmic-0.1.0" = "sha256-AAAA...";
      };
    };

    nativeBuildInputs = with final; [ pkg-config makeWrapper ];
    buildInputs = with final; [ wayland libxkbcommon vulkan-loader libGL fontconfig freetype expat ];

    postInstall = ''
      install -Dm0644 data/co.osterberg.ClaudeUsage.desktop \
        $out/share/applications/co.osterberg.ClaudeUsage.desktop
      substituteInPlace $out/share/applications/co.osterberg.ClaudeUsage.desktop \
        --replace-warn 'Exec=cosmic-applet-claude-usage' "Exec=$out/bin/cosmic-applet-claude-usage"
    '';

    postFixup = ''
      wrapProgram $out/bin/cosmic-applet-claude-usage \
        --prefix LD_LIBRARY_PATH : ${final.lib.makeLibraryPath buildInputs}
    '';

    meta = with final.lib; {
      description = "Minimal COSMIC panel applet showing Claude Code usage";
      mainProgram = "cosmic-applet-claude-usage";
      platforms = platforms.linux;
    };
  };
}

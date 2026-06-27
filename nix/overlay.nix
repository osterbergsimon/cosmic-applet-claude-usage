# nixpkgs overlay packaging this applet.
#
# Versioned alongside the source so packaging stays in sync with the code.
# To install permanently, wire it into your dotfiles flake:
#   1. import this overlay into ~/nixos-config (e.g. overlays/claude-usage),
#      pointing `src` at the checkout you build from;
#   2. add `(import ./overlays/claude-usage)` to `nixpkgs.overlays`;
#   3. add `cosmic-applet-claude-usage` to your home.packages;
#   4. run `nixos-rebuild switch`.
#
# Approach mirrors nixpkgs' own `cosmic-applets` package: a single `cargoHash`
# (fetchCargoVendor, which handles libcosmic's git workspace correctly), and
# `libcosmicAppHook`, which wraps the binary with the Wayland/Vulkan/xkb runtime
# libraries libcosmic dlopens — so no manual LD_LIBRARY_PATH wrapper is needed.
#
# Paths here are relative to this file (nix/): `src = ../.;` is the repo root.

final: prev:

{
  cosmic-applet-claude-usage = final.rustPlatform.buildRustPackage {
    pname = "cosmic-applet-claude-usage";
    version = "0.1.0";

    src = ../.;

    # fetchCargoVendor; capture with `cargoHash = ""` then copy the reported hash.
    cargoHash = "sha256-Hcf33KOzXhHbMPjOrmMsfcfOs0+tqGpNTvyJvWX3U3g=";

    __structuredAttrs = true;

    nativeBuildInputs = with final; [
      pkg-config
      libcosmicAppHook # wraps the binary with wayland/vulkan/xkb at runtime
      rustPlatform.bindgenHook
    ];

    postInstall = ''
      install -Dm0644 data/co.osterberg.ClaudeUsage.desktop \
        $out/share/applications/co.osterberg.ClaudeUsage.desktop
      substituteInPlace $out/share/applications/co.osterberg.ClaudeUsage.desktop \
        --replace-warn 'Exec=cosmic-applet-claude-usage' "Exec=$out/bin/cosmic-applet-claude-usage"
    '';

    meta = with final.lib; {
      description = "Minimal COSMIC panel applet showing Claude Code usage";
      mainProgram = "cosmic-applet-claude-usage";
      platforms = platforms.linux;
    };
  };
}

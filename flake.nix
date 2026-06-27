{
  description = "COSMIC applet showing Claude usage";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        # Native deps libcosmic/iced/winit need to build and run on Wayland.
        runtimeLibs = with pkgs; [
          wayland libxkbcommon vulkan-loader libGL
          fontconfig freetype expat
        ];
        nativeDeps = with pkgs; [ pkg-config makeWrapper ];
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = nativeDeps ++ (with pkgs; [ rustc cargo rustfmt clippy just ]);
          buildInputs = runtimeLibs;
          # winit/iced dlopen wayland & vulkan at runtime; expose them.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
          RUST_BACKTRACE = "1";
        };
      });
}

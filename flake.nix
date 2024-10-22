{
  description = "Rift Devshell";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        libraries = with pkgs;[
          rust-bin.stable.latest.default
          rust-analyzer
          trunk

          # misc. libraries
          openssl
          pkg-config

          # GUI libs
          libxkbcommon
          libGL
          fontconfig

          # wayland libraries
          wayland

          # x11 libraries
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libX11
        ];
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = libraries;

          shellHook =
          ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
          '';
        };
      }
    );
}

{
  description = "Rift Devshell";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, naersk, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = pkgs.rust-bin.stable.latest.default;
        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
          clippy = toolchain;
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
        packages.rift_egui = naersk'.buildPackage {
          pname = "rift_egui";
          src = ./.;
          cargoBuildOptions = x: x ++ [ "--package rift_egui" ];
          nativeBuildInputs = with pkgs; [
            fontconfig
          ];
        };

        packages.rift_tui = naersk'.buildPackage {
          pname = "rift_tui";
          src = ./.;
          cargoBuildOptions = x: x ++ [ "--package rift_tui" ];
        };

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

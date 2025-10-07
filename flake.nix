{
  description = "Rift Editor";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.stable.latest.default.override {
              targets = [ "wasm32-unknown-unknown" ];
          }
        );
        unfilteredRoot = ./.;
        src = pkgs.lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = pkgs.lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (pkgs.lib.fileset.fileFilter
              (file: pkgs.lib.any file.hasExt [ "html" "scss" ])
              unfilteredRoot
            )
            # folder for images, icons, etc
            (pkgs.lib.fileset.maybeMissing ./assets)
          ];
        };

        buildDeps = with pkgs; [
          (rust-bin.stable.latest.default.override {
              targets = [ "wasm32-unknown-unknown" ];
          })
          rust-analyzer
          trunk
        ];
        runtimeDeps = with pkgs; [
          makeWrapper
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
        appDeps = with pkgs; [
          fzf
          ripgrep
          fd
        ];
        devDeps = buildDeps ++ runtimeDeps ++ appDeps;

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = runtimeDeps;
          nativeBuildInputs = runtimeDeps;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          doCheck = false;
        };
        fileSetForCrate = crate: pkgs.lib.fileset.toSource {
          root = ./.;
          fileset = pkgs.lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./crates/rift_core
            ./crates/rift_egui
            ./crates/rift_tui
            ./crates/rift_rpc
            ./crates/rsl
            crate
            (pkgs.lib.fileset.fileFilter
              (file: pkgs.lib.any file.hasExt [ "html" "scss" ])
              unfilteredRoot
            )
            # folder for images, icons, etc
            (pkgs.lib.fileset.maybeMissing ./assets)
          ];
        };

        rift_tui = craneLib.buildPackage (individualCrateArgs // {
          pname = "rift_tui";
          cargoExtraArgs = "-p rift_tui";
          src = fileSetForCrate ./crates/rift_tui;
          postInstall = ''
            wrapProgram $out/bin/rift_tui \
              --prefix PATH : ${pkgs.lib.makeBinPath appDeps}
          '';
        });
        rift_egui = craneLib.buildPackage (individualCrateArgs // {
          pname = "rift_egui";
          cargoExtraArgs = "-p rift_egui";
          src = fileSetForCrate ./crates/rift_egui;
          postInstall = ''
            wrapProgram $out/bin/rift_egui \
              --prefix PATH : ${pkgs.lib.makeBinPath appDeps} \
              --set LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeDeps}
          '';
        });
      in
      {
        packages = {
          inherit rift_tui rift_egui;
        };

        apps = {
          rift_tui = flake-utils.lib.mkApp {
            drv = rift_tui;
          };
          rift_egui = flake-utils.lib.mkApp {
            drv = rift_egui;
          };
        };

        devShells.default = craneLib.devShell {
          packages = devDeps;

          shellHook =
          ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath devDeps}:$LD_LIBRARY_PATH
          '';
        };
      }
    );
}

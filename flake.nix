{
  description = "Rift Editor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        buildDeps = with pkgs; [
          toolchain
          trunk
          nixfmt-rfc-style
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

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        unfilteredRoot = ./.;
        src = pkgs.lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = pkgs.lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (pkgs.lib.fileset.fileFilter (
              file:
              pkgs.lib.any file.hasExt [
                "rsl"
                "html"
                "scss"
              ]
            ) unfilteredRoot)
            # folder for images, icons, etc
            (pkgs.lib.fileset.maybeMissing ./assets)
          ];
        };

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
        };

        rift_tui = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rift_tui";
            cargoExtraArgs = "-p rift_tui";
            postInstall = ''
              wrapProgram $out/bin/rt \
                --prefix PATH : ${pkgs.lib.makeBinPath appDeps}
            '';
          }
        );
        rift_egui = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rift_egui";
            cargoExtraArgs = "-p rift_egui";
            postInstall = ''
              wrapProgram $out/bin/re \
                --prefix PATH : ${pkgs.lib.makeBinPath appDeps} \
                --set LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeDeps}
            '';
          }
        );
      in
      {
        checks = {
          inherit rift_tui rift_egui;

          rift-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          rift-fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        packages = {
          inherit rift_tui rift_egui;
        };

        apps = {
          rift_tui = flake-utils.lib.mkApp {
            drv = rift_tui;
            exePath = "/bin/rt";
          };
          rift_egui = flake-utils.lib.mkApp {
            drv = rift_egui;
            exePath = "/bin/re";
          };
        };

        devShells.default = craneLib.devShell {
          packages = devDeps;

          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath devDeps}:$LD_LIBRARY_PATH
          '';
        };
      }
    );
}

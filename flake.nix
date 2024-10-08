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
          webkitgtk
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          librsvg

          
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

        packages = with pkgs; [
          curl
          wget
          pkg-config
          dbus
          openssl
          glib
          gtk3
          libsoup
          webkitgtk
          librsvg

          rust-bin.stable.latest.default
          rust-analyzer
          typescript-language-server
          vue-language-server
          vscode-langservers-extracted
          tailwindcss-language-server
          nodejs_20
          cargo-tauri
        ];
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = packages;

          shellHook =
          ''
            # export WEBKIT_DISABLE_COMPOSITING_MODE=1
            export WEBKIT_DISABLE_DMABUF_RENDERER=1
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
            export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
          '';
        };
      }
    );
}

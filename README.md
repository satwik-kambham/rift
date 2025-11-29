# Rift Editor

Extensible modal code editor inspired by emacs, neovim and helix.

![Rift editor screenshot](rift.png)

## Try it out!

GUI Frontend:
```
  nix run github:satwik-kambham/rift#rift_egui
```

TUI Frontend:
```
  nix run github:satwik-kambham/rift#rift_tui
```

## Features

- Modal Editing
- Tree sitter syntax highlighting
- LSP Integration

## Documentation

- RSL quick start: [docs/rsl-quickstart.md](docs/rsl-quickstart.md)
- Additional guides live in the `docs/` directory.

## Build instructions

1. Install rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Proceed with standard installation.
Additionally add `rust-analyzer` as a component if desired:
```
rustup component add rust-analyzer
```

2. Install egui dependencies on linux

Debian based distros:
```
sudo apt install build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libssl-dev libfontconfig-dev
```

Fedora:
```
dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel
```

3. Install optional application dependencies

- ripgrep
- fzf
- fd

4. Build application

```
cargo b -r
```

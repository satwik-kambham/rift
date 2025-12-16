# Build from source

Follow these steps to compile Rift locally.

1. Install Rust with rustup:
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   Proceed with the standard installation. Add `rust-analyzer` if you want IDE support:
   ```
   rustup component add rust-analyzer
   ```

2. Install egui system dependencies on Linux:

   - Debian-based:
     ```
     sudo apt install build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libssl-dev libfontconfig-dev
     ```
   - Fedora:
     ```
     dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel
     ```

3. (Optional) Install application helpers: ripgrep, fzf, fd.

4. Build the project:
   ```
   cargo b -r
   ```

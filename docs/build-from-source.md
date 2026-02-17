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

2. (Optional) Install application helpers: ripgrep, fzf, fd.

3. Build the project:
   ```
   cargo b -r
   ```

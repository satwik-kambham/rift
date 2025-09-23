# Rift

## Project Overview

Rift is an code editor written in rust with two frontends

## Code Structure

The project is organized into multiple crates within the `crates/` directory:

- `rift_core`: Core functionality
- `rift_tui`: TUI frontend using ratatui
- `rift_egui`: GUI frontend using egui
- `rsl`: Rift Scripting Language implentation

The project developement environment is create using nix devshell.

- `flake.nix` contains devshell and build code

## Build Instructions and Useful Commands

- Check for errors: `cargo c`
- Lint code: `cargo clippy`
- Format code: `cargo fmt`
- Run tui frontend: `cargo b -p rift_tui`
- Run egui frontend: `cargo b -p rift_egui`
- Build all executables for release: `cargo b -r`

## Important Files

- `crates/rift_core/src/state.rs` - Main editor state
- `crates/rift_egui/src/app.rs` - Main egui code
- `crates/rift_tui/src/app.rs` - main tui code

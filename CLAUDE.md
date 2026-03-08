# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rift is an extensible modal code editor (Normal/Insert modes) written in Rust, inspired by emacs, neovim, and helix. It features tree-sitter syntax highlighting, LSP integration, an embedded scripting language (RSL), an agentic coding assistant, and optional voice control.

## Build & Development Commands

```bash
# Dev environment (Nix)
nix develop          # or: direnv allow

# Build
cargo build          # debug build
cargo build -r       # release build

# Run
cargo run -p rift_tui --release              # TUI frontend
cargo run -p rift_tui --release -- --no-lsp --no-audio  # minimal (no LSP/audio)
cargo run -p rift_server --release -- --no-audio         # web server frontend

# Lint & format
cargo fmt
cargo clippy -- -D warnings

# Profiling
cargo flamegraph -p rift_tui
```

The TUI binary is named `rt`. Rust toolchain version is pinned in `rust-toolchain.toml` (currently 1.93.0, edition 2024).

## Architecture

**Workspace crates** (all under `crates/`):

- **`rift_core`** — Editor primitives: state (`EditorState`), buffers (`RopeBuffer`/`BufferInstance`), actions, LSP client, keybind handling, rendering, audio, and the RSL bridge. This is the shared core used by all frontends.
- **`rift_tui`** — Terminal frontend using ratatui. Entry point creates `App`, initializes terminal, and runs the main loop.
- **`rift_server`** — Web frontend using axum + WebSocket + JS. Serves static files from `static/` and communicates via `rift_ipc` message types.
- **`rift_rpc`** — Defines the `RiftRPC` tarpc service trait. This is the RPC interface between `rift_core` and RSL scripts (buffer ops, keybinds, LSP queries, actions).
- **`rift_ipc`** — Binary protocol types (`ClientToServer`/`ServerToClient`) for the WebSocket server frontend, using bincode.
- **`rsl`** — The Rift Scripting Language: scanner, parser, interpreter, environment, and standard library. Uses `inventory` crate for native function registration. RSL scripts (`.rsl` files) provide editor UI wiring and workflows.
- **`rsl_macros`** — Proc macros for RSL.

**Key data flow:**
1. Frontends drive the main loop, polling input and calling into `rift_core`.
2. `EditorState` owns all buffers, LSP handles, and the tokio runtime.
3. Actions are dispatched via `perform_action(Action, &mut EditorState)` in `actions.rs`.
4. RSL scripts call editor functionality through tarpc RPC (`rift_rpc` trait). The RSL interpreter runs in a separate thread, communicating via channels.
5. LSP requests are fire-and-forget; responses are handled in `handle_lsp_messages` once per frame, which updates state and triggers RSL UI entry points.

**LSP flow:** Send requests via `lsp_handle.lock().unwrap().send_request_sync(...)` — do not block or poll. Responses arrive in the frame-driven `handle_lsp_messages` pump. Currently supports rust-analyzer, pylsp (via uv), dart language server, and zls.

## Coding Conventions

- Rustfmt defaults (4-space indent, 100-char line width); always run `cargo fmt`.
- Use `tracing` for diagnostics, not `println!`.
- Prefer returning errors over panicking; reserve `panic!`/`expect` for truly unrecoverable situations.
- RSL `.rsl` files use lowerCamelCase naming.
- Commit format: `<type>: <short imperative summary>` (types: `feat`, `fix`, `docs`, `refactor`, `chore`, `test`, `build`).

## Runtime Dependencies

The editor expects these tools on `PATH` (provided by the Nix devshell): `fzf`, `ripgrep` (`rg`), `fd`, `ffmpeg`. Optional voice control requires the Synapse API running on port 8000.

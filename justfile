# Common workflows for the Rift workspace.
# Run `just` or `just --list` to see available recipes.

# List available recipes
default:
    @just --list

# Quick compile check
check:
    cargo check
    cargo clippy

# Full CI: compile, clippy (deny warnings), fmt check, and tests
check-all:
    cargo check
    cargo clippy -- -D warnings
    cargo fmt -- --check
    cargo test -p rsl
    cargo test --workspace --exclude rsl
    @echo "All checks passed."

# Format the workspace
fmt:
    cargo fmt

# Lint with warnings denied
clippy:
    cargo clippy -- -D warnings

# Debug build
build:
    cargo build

# Release build
build-release:
    cargo build -r

# Run the TUI frontend
run-tui:
    cargo run -p rift_tui --release

# Run the TUI without LSP or audio
run-tui-minimal:
    cargo run -p rift_tui --release -- --no-lsp --no-audio

# Flamegraph the TUI
flamegraph-tui:
    cargo flamegraph -p rift_tui

# Flamegraph the TUI with debug symbols
flamegraph-tui-debug:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_tui

# Run all tests
test:
    cargo test --workspace

# Remove build artifacts
clean:
    cargo clean

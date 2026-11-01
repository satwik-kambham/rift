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

# Bump rust-toolchain.toml to the latest stable release (requires curl)
update-toolchain:
    #!/usr/bin/env bash
    set -euo pipefail
    manifest=$(mktemp)
    trap 'rm -f "$manifest" rust-toolchain.toml.tmp' EXIT
    # Download to a file rather than piping: awk exits at the first match, which
    # would close the pipe mid-download and make curl fail with "Failure writing
    # output to destination".
    curl -fsSL -o "$manifest" https://static.rust-lang.org/dist/channel-rust-stable.toml
    latest=$(awk -F'"' '/^\[pkg\.rust\]/{found=1} found && /^version = /{split($2, parts, " "); print parts[1]; exit}' "$manifest")
    if [ -z "${latest:-}" ]; then
        echo "error: could not resolve latest stable Rust version" >&2
        exit 1
    fi
    current=$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml)
    if [ "$current" = "$latest" ]; then
        echo "rust-toolchain.toml already at $latest"
        exit 0
    fi
    sed "s/^channel = .*/channel = \"$latest\"/" rust-toolchain.toml > rust-toolchain.toml.tmp
    mv rust-toolchain.toml.tmp rust-toolchain.toml
    echo "rust-toolchain.toml: $current -> $latest"

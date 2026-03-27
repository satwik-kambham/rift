.PHONY: help check check-all fmt clippy build build-release run-tui run-tui-release \
	flamegraph-tui flamegraph-tui-debug clean run-tui-minimal test

help:
	@echo "Common targets:"
	@echo "  check            - cargo check; cargo clippy"
	@echo "  fmt              - cargo fmt"
	@echo "  clippy           - cargo clippy -- -D warnings"
	@echo "  build            - cargo build"
	@echo "  build-release    - cargo build -r"
	@echo "  run-tui          - cargo run -p rift_tui --release"
	@echo "  run-tui-minimal  - cargo run -p rift_tui --release -- --no-lsp --no-audio"
	@echo "  run-server       - cargo run -p rift_server --release -- --no-audio"
	@echo "  flamegraph-tui   - cargo flamegraph -p rift_tui"
	@echo "  flamegraph-tui-debug  - CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_tui"
	@echo "  test             - cargo test --workspace"
	@echo "  check-all        - cargo check, clippy, fmt check, and test"
	@echo "  clean            - cargo clean"

check:
	cargo check; cargo clippy

check-all:
	cargo check
	cargo clippy -- -D warnings
	cargo fmt -- --check
	cargo test -p rsl
	cargo test --workspace --exclude rsl
	@echo ""
	@echo "All checks passed."

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

build:
	cargo build

build-release:
	cargo build -r

run-tui:
	cargo run -p rift_tui --release

run-tui-minimal:
	cargo run -p rift_tui --release -- --no-lsp --no-audio


run-server:
	cargo run -p rift_server --release -- --no-audio

flamegraph-tui:
	cargo flamegraph -p rift_tui

flamegraph-tui-debug:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_tui

test:
	cargo test --workspace

clean:
	cargo clean

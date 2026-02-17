.PHONY: help check fmt clippy build build-release run-tui run-tui-release \
	flamegraph-tui flamegraph-tui-debug clean

help:
	@echo "Common targets:"
	@echo "  check            - cargo check; cargo clippy"
	@echo "  fmt              - cargo fmt"
	@echo "  clippy           - cargo clippy -- -D warnings"
	@echo "  build            - cargo build"
	@echo "  build-release    - cargo build -r"
	@echo "  run-tui          - cargo run -p rift_tui"
	@echo "  run-tui-release  - cargo run -p rift_tui --release"
	@echo "  flamegraph-tui   - cargo flamegraph -p rift_tui"
	@echo "  flamegraph-tui-debug  - CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_tui"
	@echo "  clean            - cargo clean"

check:
	cargo check; cargo clippy

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

build:
	cargo build

build-release:
	cargo build -r

run-tui:
	cargo run -p rift_tui

run-tui-release:
	cargo run -p rift_tui --release

flamegraph-tui:
	cargo flamegraph -p rift_tui

flamegraph-tui-debug:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_tui

clean:
	cargo clean

.PHONY: help check fmt clippy build build-release run-tui run-egui run-tui-release \
	run-egui-release flamegraph-egui flamegraph-tui flamegraph-egui-debug \
	flamegraph-tui-debug clean

help:
	@echo "Common targets:"
	@echo "  check            - cargo check"
	@echo "  fmt              - cargo fmt"
	@echo "  clippy           - cargo clippy -- -D warnings"
	@echo "  build            - cargo build"
	@echo "  build-release    - cargo build -r"
	@echo "  run-tui          - cargo run -p rift_tui"
	@echo "  run-egui         - cargo run -p rift_egui"
	@echo "  run-tui-release  - cargo run -p rift_tui --release"
	@echo "  run-egui-release - cargo run -p rift_egui --release"
	@echo "  flamegraph-egui  - cargo flamegraph -p rift_egui"
	@echo "  flamegraph-tui   - cargo flamegraph -p rift_tui"
	@echo "  flamegraph-egui-debug - CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_egui"
	@echo "  flamegraph-tui-debug  - CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_tui"
	@echo "  clean            - cargo clean"

check:
	cargo check

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

run-egui:
	cargo run -p rift_egui

run-tui-release:
	cargo run -p rift_tui --release

run-egui-release:
	cargo run -p rift_egui --release

flamegraph-egui:
	cargo flamegraph -p rift_egui

flamegraph-tui:
	cargo flamegraph -p rift_tui

flamegraph-egui-debug:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_egui

flamegraph-tui-debug:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p rift_tui

clean:
	cargo clean

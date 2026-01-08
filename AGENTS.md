# Repository Guidelines

## Project Structure & Module Organization
- Rust workspace rooted at this repo; crates live under `crates/`.
- `crates/rift_core` holds editor primitives, buffer/state, and shared utilities.
- Frontends: `crates/rift_tui` (ratatui), `crates/rift_egui` (egui GUI), and `crates/rift_web` (egui + wasm + trunk).
- `crates/rift_server` is an in-progress backend host; current frontends do not rely on it yet.
- `crates/rift_rpc` provides RPC between `rift_core` and RSL.
- `crates/rsl` contains the Rift scripting language implementation.
- Markdown documentation lives under `docs/` at the repo root.
- Architecture overview and LSP flow details live in `docs/architecture.md`.
- Assets such as screenshots live in `assets/`; build artifacts land in `target/`.
- A Nix devshell is defined in `flake.nix`; with direnv allowed, `.envrc` auto-loads the devshell by default.

## Build & Development Commands
- Enter dev env: `nix develop` (or `direnv allow` in this repo).
- Fast feedback: `cargo check` to validate the workspace compiles.
- Lint/format: `cargo fmt` then `cargo clippy -- -D warnings`.
- Run frontends: `cargo run -p rift_tui` for the TUI, `cargo run -p rift_egui` for the GUI; add `--release` for performance testing.
- Full release build: `cargo build -r` (or `cargo build -r -p rift_tui` to target one binary).
- Web builds use the wasm32 target; it's preinstalled in the Nix devshell. If you are outside the devshell, add it via `rustup target add wasm32-unknown-unknown` and install trunk/wasm-bindgen as needed.

## Coding Style & Naming Conventions

### Rust
- Use Rustfmt defaults (4-space indent, 100-line wraps); always run `cargo fmt`.
- Prefer idiomatic naming: `snake_case` for functions/vars/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for consts.
- Keep modules focused; mirror existing patterns in `rift_core` (commands/state) and frontends (UI wiring).
- Prefer `tracing` diagnostics over `println!`.

### RSL
- `.rsl` files contain Rift scripting language code; naming uses lowerCamelCase.
- Infer other conventions from existing file contents; match surrounding style and patterns.

## Error Handling
- Prefer returning errors over panicking; reserve `panic!`/`expect` for truly unrecoverable situations.
- Use `tracing` for diagnostics in Rust code; include context (paths, ids) when logging errors.
- When RSL scripts need to emit diagnostics, send them through the `log` RPC function with helpful context.
- For recoverable operations (file IO, parsing, RPC calls), avoid `unwrap`/`expect`; propagate errors or fallback gracefully where possible.
- If an error is handled locally, log it at an appropriate level (`warn` for degraded behavior, `error` for failed operations) and continue when safe.
- Convert external errors into meaningful application errors so callers can decide whether to retry, degrade, or surface to the user.

## Commit & Pull Request Guidelines
- Commit message format:
  - `<type>: <short imperative summary>`; types: `feat`, `fix`, `docs`, `refactor`, `chore`, `test`, `build`.
  - Blank line after the subject.
  - Body wrapped at ~72 chars; focus on what and why, not step-by-step how.
  - Footer for references or breaking changes, e.g., `Refs: #123`, `BREAKING CHANGE: ...`.
- Before opening a PR, run `cargo fmt` and `cargo clippy -- -D warnings`; mention anything not run and why.
- PRs should describe scope, motivations, and user-facing impact; link issues when applicable. Include screenshots or terminal captures for UI/UX changes (TUI/GUI/web).
- Keep changes scoped to one concern; split large refactors from feature work when possible. Document new flags/config in README or crate-level docs.

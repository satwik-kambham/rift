# Repository Guidelines

## Project Structure

Rust workspace rooted at this repo; crates live under `crates/`.

| Crate | Purpose |
|-------|---------|
| `rift_core` | Editor primitives, buffer/state, and shared utilities |
| `rift_tui` | Terminal frontend (ratatui) |
| `rift_server` | Web frontend (axum + js) |
| `rift_rpc` | RPC bridge between `rift_core` and RSL |
| `rift_ipc` | IPC protocol types (key input, client/server messages) |
| `rsl` | Rift scripting language implementation |
| `rsl_macros` | Proc macros for registering native RSL functions |
| `petal` | Note store library |
| `petal_mcp` | MCP server exposing the petal note store |

Other paths:
- `docs/` — documentation (`architecture.md`, `petal.md`, `rsl-quickstart.md`).
- `assets/` — screenshots and static assets; `target/` — build artifacts.
- `flake.nix` + `.envrc` — Nix devshell; `direnv allow` auto-loads it.

## Build & Development Commands

A `Makefile` wraps common workflows — run `make help` to list all targets.

| Task | Command |
|------|---------|
| Enter dev env | `nix develop` (or `direnv allow`) |
| Quick compile check | `make check` (`cargo check` + `cargo clippy`) |
| Full CI check | `make check-all` (check, clippy `-D warnings`, fmt `--check`, tests) |
| Format | `make fmt` |
| Lint | `make clippy` (`cargo clippy -- -D warnings`) |
| Tests | `make test` (`cargo test --workspace`) |
| Run TUI | `make run-tui` (release) |
| Run TUI (minimal) | `make run-tui-minimal` (`--no-lsp --no-audio`) |
| Run server | `make run-server` (`--no-audio`) |
| Release build | `make build-release` |
| Flamegraph | `make flamegraph-tui` / `make flamegraph-tui-debug` |
| Clean | `make clean` |

## Coding Style

### Rust
- Use Rustfmt defaults (4-space indent, 100-line wraps); always run `make fmt`.
- Naming: `snake_case` for functions/vars/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for consts.
- Mirror existing patterns in `rift_core` (commands/state) and frontends (UI wiring).
- Prefer `tracing` diagnostics over `println!`.

### RSL
- `.rsl` files use lowerCamelCase naming.
- Match surrounding style; refer to `docs/rsl-quickstart.md` for language docs and the `rsl` crate for implementation details.

## Error Handling
- Prefer returning errors over panicking; reserve `panic!`/`expect` for truly unrecoverable situations.
- Use `tracing` for diagnostics; include context (paths, ids) when logging errors.
- When RSL scripts need to emit diagnostics, send them through the `log` RPC function with helpful context.
- Avoid `unwrap`/`expect` on recoverable operations (file IO, parsing, RPC); propagate errors or fall back gracefully.
- Log handled errors at appropriate levels (`warn` for degraded behavior, `error` for failures).
- Convert external errors into meaningful application errors so callers can decide to retry, degrade, or surface to the user.

## Commit & Pull Request Guidelines
- Format: `<type>: <short imperative summary>` — types: `feat`, `fix`, `docs`, `refactor`, `chore`, `test`, `build`.
- Blank line after subject; body wrapped at ~72 chars; focus on what and why.
- Footer for references or breaking changes, e.g., `Refs: #123`, `BREAKING CHANGE: ...`.

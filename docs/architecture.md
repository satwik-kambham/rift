# Architecture

## Overview
Rift uses a Rust core (`crates/rift_core`) that owns editor state, buffers, and the LSP client. Frontends (TUI/egui/wasm) render the state and drive input. RSL scripts provide UI wiring and editor workflows by calling into RPC methods exposed by the core.

## LSP

### Sending requests
LSP requests and notifications are sent through the shared LSP client handle. Callers should only enqueue a request and return immediately; they should not block or poll for responses.

Example request:
```
lsp_handle
    .lock()
    .unwrap()
    .send_request_sync(
        "textDocument/references".to_string(),
        Some(LSPClientHandle::go_to_references_request(
            file_path.clone(),
            cursor,
        )),
    );
```

### Handling responses and notifications
Incoming LSP messages are handled only in `handle_lsp_messages` and are polled once per frame by the frontends. Do not call `handle_lsp_messages` from actions or other code paths.

When a response should drive UI (e.g., definitions, references, completions), the handler must update editor state and then trigger the appropriate RSL UI entry point (e.g., `createGoToDefinition()` or `createGoToReferences()`). This mirrors the existing completion flow and keeps the UI update tied to the frame-driven message pump.

### Go To Definition / References flow
- Action sends `textDocument/definition` or `textDocument/references` and returns immediately.
- `handle_lsp_messages` receives the response, populates `state.definitions` or `state.references`, and triggers the RSL UI to render the results.
- RSL `getDefinitions()` / `getReferences()` return the cached results for UI rendering and filtering.

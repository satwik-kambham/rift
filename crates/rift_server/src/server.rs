use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use rift_core::{
    actions::{Action, perform_action},
    buffer::instance::{Attribute, Language},
    cli::{CLIArgs, process_cli_args},
    io::file_io::handle_file_event,
    lsp::{client::LSPClientHandle, handle_lsp_messages},
    preferences::Color,
    rendering::update_visible_lines,
    rsl::initialize_rsl,
    state::{CompletionMenu, EditorState, Mode},
};

pub struct Server {
    pub state: EditorState,
    pub lsp_handles: HashMap<Language, LSPClientHandle>,
}

impl Server {
    pub fn new(rt: tokio::runtime::Runtime, cli_args: CLIArgs) -> Self {
        let mut state = EditorState::new(rt);
        let mut lsp_handles = HashMap::new();

        process_cli_args(cli_args, &mut state, &mut lsp_handles);

        initialize_rsl(&mut state, &mut lsp_handles);

        Self { state, lsp_handles }
    }

    pub fn perform_action(&mut self, action: Action) -> String {
        perform_action(action, &mut self.state, &mut self.lsp_handles).unwrap_or_default()
    }

    pub fn run(&mut self) {
        while !self.state.quit {
            // Run async callbacks
            if let Ok(async_result) = self.state.async_handle.receiver.try_recv() {
                (async_result.callback)(
                    async_result.result,
                    &mut self.state,
                    &mut self.lsp_handles,
                );
                self.state.update_view = true;
            }

            // Run action requests
            if let Ok(action_request) = self.state.event_reciever.try_recv() {
                let result = self.perform_action(action_request.action);
                action_request.response_tx.send(result).unwrap();
                self.state.update_view = true;
            }

            // Handle file watcher events
            if let Ok(file_event_result) = self.state.file_event_receiver.try_recv() {
                handle_file_event(file_event_result, &mut self.state, &mut self.lsp_handles);
                self.state.update_view = true;
            }

            // Handle lsp messages
            handle_lsp_messages(&mut self.state, &mut self.lsp_handles);
        }
    }
}

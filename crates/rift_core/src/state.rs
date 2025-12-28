use std::{collections::HashMap, path::Path};

use clap::Parser;
use copypasta::ClipboardContext;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use tokio::sync::mpsc;

use crate::{
    actions::{Action, ReferenceEntry, perform_action},
    buffer::{
        instance::{BufferInstance, Cursor, GutterInfo, Language},
        rope_buffer::{HighlightedText, RopeBuffer},
    },
    cli::CLIArgs,
    concurrent::{AsyncHandle, AsyncResult},
    keybinds::KeybindHandler,
    lsp::{
        client::{LSPClientHandle, start_lsp},
        types,
    },
    preferences::Preferences,
    rpc::{RPCRequest, start_rpc_server},
    rsl::start_rsl_interpreter,
};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

pub struct EditorState {
    pub preferences: Preferences,
    pub buffers: HashMap<u32, RopeBuffer>,
    pub instances: HashMap<u32, BufferInstance>,
    next_id: u32,
    pub workspace_folder: String,
    pub current_folder: String,
    pub mode: Mode,
    pub buffer_idx: Option<u32>,
    pub log_messages: Vec<String>,
    pub register: String,
    pub search_query: String,
    pub quit: bool,

    // System
    pub clipboard_ctx: Option<ClipboardContext>,

    // Handles
    pub rt: tokio::runtime::Runtime,
    pub async_handle: AsyncHandle,
    pub file_event_receiver: mpsc::Receiver<NotifyResult<Event>>,
    pub event_reciever: mpsc::Receiver<RPCRequest>,
    pub rsl_sender: mpsc::Sender<String>,
    pub file_watcher: Option<RecommendedWatcher>,
    lsp_handles: HashMap<Language, LSPClientHandle>,

    // LSP
    pub diagnostics: HashMap<String, types::PublishDiagnostics>,
    pub references: Vec<ReferenceEntry>,
    pub references_version: usize,
    pub definitions: Vec<ReferenceEntry>,
    pub definitions_version: usize,
    pub diagnostics_overlay: DiagnosticsOverlay,

    // UI
    viewport_rows: usize,
    viewport_columns: usize,
    pub update_view: bool,
    pub highlighted_text: HighlightedText,
    pub gutter_info: Vec<GutterInfo>,
    pub relative_cursor: Cursor,
    pub completion_menu: CompletionMenu,
    pub signature_information: SignatureInformation,
    pub keybind_handler: KeybindHandler,
}

impl EditorState {
    pub fn new() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cli_args = CLIArgs::parse();

        // if let Err(err) = process_cli_args(cli_args, &mut state, &mut lsp_handles) {
        // error!(%err, "Failed to process CLI args");
        // }

        let (event_sender, event_reciever) = mpsc::channel::<RPCRequest>(32);

        let rpc_client_transport = rt.block_on(start_rpc_server(event_sender));

        let (sender, receiver) = mpsc::channel::<AsyncResult>(32);
        let (file_event_sender, file_event_receiver) = mpsc::channel::<NotifyResult<Event>>(32);

        let rt_handle = rt.handle().clone();
        let watcher = RecommendedWatcher::new(
            move |res| {
                rt_handle.block_on(async {
                    if let Err(err) = file_event_sender.clone().send(res).await {
                        tracing::warn!(%err, "Failed to forward file watcher event");
                    }
                });
            },
            Config::default(),
        )
        .map(Some)
        .unwrap_or_else(|err| {
            tracing::warn!(%err, "Failed to create file watcher; continuing without live file updates");
            None
        });

        let initial_folder =
            std::path::absolute("/").unwrap_or_else(|_| std::path::PathBuf::from("/"));
        let initial_folder_str = initial_folder.to_string_lossy().to_string();

        let rsl_sender = start_rsl_interpreter(initial_folder_str.clone(), rpc_client_transport);

        Self {
            quit: false,
            rt,
            async_handle: AsyncHandle { sender, receiver },
            file_event_receiver,
            event_reciever,
            rsl_sender,
            file_watcher: watcher,
            preferences: Preferences::default(),
            buffers: HashMap::new(),
            next_id: 0,
            workspace_folder: initial_folder_str.clone(),
            current_folder: initial_folder_str,
            viewport_rows: 0,
            viewport_columns: 0,
            mode: Mode::Normal,
            instances: HashMap::new(),
            highlighted_text: vec![],
            gutter_info: vec![],
            buffer_idx: None,
            relative_cursor: Cursor { row: 0, column: 0 },
            update_view: true,
            clipboard_ctx: ClipboardContext::new().ok(),
            diagnostics: HashMap::new(),
            references: vec![],
            references_version: 0,
            definitions: vec![],
            definitions_version: 0,
            diagnostics_overlay: DiagnosticsOverlay::default(),
            completion_menu: CompletionMenu::new(5),
            signature_information: SignatureInformation::default(),
            keybind_handler: KeybindHandler::default(),
            log_messages: vec![],
            register: String::new(),
            search_query: String::new(),
            lsp_handles: HashMap::new(),
        }
    }

    pub fn viewport_rows(&self) -> usize {
        self.viewport_rows
    }

    pub fn viewport_columns(&self) -> usize {
        self.viewport_columns
    }

    pub fn set_viewport_size(&mut self, rows: usize, columns: usize) -> bool {
        let changed = self.viewport_rows != rows || self.viewport_columns != columns;
        if changed {
            self.viewport_rows = rows;
            self.viewport_columns = columns;
        }
        changed
    }

    pub fn add_buffer(&mut self, buffer: RopeBuffer) -> u32 {
        if let Some((idx, _)) = self
            .buffers
            .iter()
            .find(|(_, buf)| !buf.special && buf.file_path() == buffer.file_path())
        {
            *idx
        } else {
            if let Some(path_str) = buffer.file_path() {
                let path = Path::new(path_str);
                if let Some(watcher) = self.file_watcher.as_mut()
                    && let Err(err) = watcher.watch(path, RecursiveMode::NonRecursive)
                {
                    tracing::warn!(%err, path = %path.display(), "Failed to watch file path");
                }
            }
            self.buffers.insert(self.next_id, buffer);
            self.instances
                .insert(self.next_id, BufferInstance::new(self.next_id));
            self.next_id += 1;
            self.next_id - 1
        }
    }

    pub fn remove_buffer(&mut self, id: u32) {
        self.buffers.remove(&id);
        if self.buffers.is_empty() {
            self.buffer_idx = None;
        } else {
            self.buffer_idx = Some(self.buffer_idx.unwrap().saturating_sub(1));
        }
    }

    pub fn cycle_buffer(&mut self, reverse: bool, regular_only: bool) {
        let Some(current_id) = self.buffer_idx else {
            return;
        };

        let mut buffer_ids: Vec<u32> = self.buffers.keys().copied().collect();
        if buffer_ids.is_empty() {
            return;
        }

        buffer_ids.sort_unstable();

        let Some(mut position) = buffer_ids.iter().position(|id| *id == current_id) else {
            return;
        };

        let len = buffer_ids.len();
        for _ in 0..len {
            position = if reverse {
                position.checked_sub(1).unwrap_or(len - 1)
            } else {
                (position + 1) % len
            };

            let candidate_id = buffer_ids[position];
            if let Some(buffer) = self.buffers.get(&candidate_id)
                && (!regular_only || !buffer.special)
            {
                self.buffer_idx = Some(candidate_id);
                return;
            }
        }
    }

    pub fn get_buffer_by_id(&self, id: u32) -> (&RopeBuffer, &BufferInstance) {
        (
            self.buffers.get(&id).unwrap(),
            self.instances.get(&id).unwrap(),
        )
    }

    pub fn get_buffer_by_id_mut(&mut self, id: u32) -> (&mut RopeBuffer, &mut BufferInstance) {
        (
            self.buffers.get_mut(&id).unwrap(),
            self.instances.get_mut(&id).unwrap(),
        )
    }

    pub fn is_active_buffer_special(&self) -> Option<bool> {
        if let Some(buffer_idx) = self.buffer_idx {
            if let Some(buffer) = self.buffers.get(&buffer_idx) {
                return Some(buffer.special);
            }
            return None;
        }
        None
    }

    pub fn spawn_lsp(&self, language: Language) -> Option<LSPClientHandle> {
        if self.preferences.no_lsp {
            return None;
        }

        let command: Option<(&str, &[&str])> = match language {
            Language::Rust => Some(("rust-analyzer", &[])),
            Language::Python => Some(("uv", &["run", "pylsp"])),
            Language::Dart => Some(("dart", &["language-server", "--client-id=rift"])),
            // Language::HTML => Some(("vscode-html-language-server", &["--stdio"])),
            // Language::CSS => Some(("vscode-css-language-server", &["--stdio"])),
            // Language::JSON => Some(("vscode-json-language-server", &["--stdio"])),
            // Language::Javascript => Some(("typescript-language-server", &["--stdio"])),
            // Language::Typescript => Some(("typescript-language-server", &["--stdio"])),
            // Language::Tsx => Some(("typescript-language-server", &["--stdio"])),
            // Language::Vue => Some(("vue-language-server", &["--stdio"])),
            _ => None,
        };
        if let Some(command) = command {
            if which::which(command.0).is_ok() {
                match self
                    .rt
                    .block_on(async { start_lsp(command.0, command.1).await })
                {
                    Ok(client) => return Some(client),
                    Err(err) => {
                        let command_display = if command.1.is_empty() {
                            command.0.to_string()
                        } else {
                            format!("{} {}", command.0, command.1.join(" "))
                        };
                        tracing::error!("Failed to start LSP `{}`: {err}", command_display);
                    }
                }
            } else {
                return None;
            }
        }
        None
    }
}

pub struct DiagnosticsOverlay {
    pub content: String,
}

impl DiagnosticsOverlay {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn should_render(&self) -> bool {
        !self.content.is_empty()
    }
}

impl Default for DiagnosticsOverlay {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SignatureInformation {
    pub content: String,
}

impl SignatureInformation {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn should_render(&self) -> bool {
        !self.content.is_empty()
    }
}

impl Default for SignatureInformation {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompletionMenu {
    pub active: bool,
    pub items: Vec<types::CompletionItem>,
    pub start: usize,
    pub selection: Option<usize>,
    pub max_items: usize,
}

impl CompletionMenu {
    pub fn new(max_items: usize) -> Self {
        Self {
            active: false,
            items: vec![],
            start: 0,
            selection: None,
            max_items,
        }
    }

    pub fn open(&mut self, items: Vec<types::CompletionItem>) {
        if !items.is_empty() {
            self.active = true;
            self.items = items;
            self.start = 0;
            self.selection = None;
        }
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selection = if let Some(idx) = self.selection {
                if idx < self.items.len() - 1 {
                    self.start += 1;
                    Some(idx + 1)
                } else {
                    self.start = 0;
                    Some(0)
                }
            } else {
                self.start = 0;
                Some(0)
            };
        }
    }

    pub fn select(&mut self) -> Option<types::CompletionItem> {
        if let Some(idx) = self.selection {
            self.close();
            return Some(self.items[idx].clone());
        }
        self.close();
        None
    }

    pub fn on_select(
        completion_item: Option<types::CompletionItem>,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        if let Some(completion_item) = completion_item {
            perform_action(
                Action::DeleteText(completion_item.edit.range),
                state,
                lsp_handles,
            );
            perform_action(
                Action::InsertText(
                    completion_item.edit.text.clone(),
                    completion_item.edit.range.mark,
                ),
                state,
                lsp_handles,
            );
        }
    }

    pub fn close(&mut self) {
        self.active = false;
    }
}

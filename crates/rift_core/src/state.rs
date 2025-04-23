use std::{collections::HashMap, path::Path};

use copypasta::ClipboardContext;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use tokio::sync::mpsc;

use crate::{
    actions::{perform_action, Action},
    ai::AIState,
    buffer::{
        instance::{BufferInstance, Cursor, GutterInfo, Language},
        line_buffer::{HighlightedText, LineBuffer},
    },
    concurrent::{AsyncHandle, AsyncResult},
    keybinds::KeybindHandler,
    lsp::{
        client::{start_lsp, LSPClientHandle},
        types,
    },
    preferences::Preferences,
};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

pub struct EditorState {
    pub quit: bool,
    pub rt: tokio::runtime::Runtime,
    pub async_handle: AsyncHandle,
    pub file_event_receiver: mpsc::Receiver<NotifyResult<Event>>,
    pub file_watcher: RecommendedWatcher,
    pub preferences: Preferences,
    pub buffers: HashMap<u32, LineBuffer>,
    pub instances: HashMap<u32, BufferInstance>,
    next_id: u32,
    pub workspace_folder: String,
    pub current_folder: String,
    pub visible_lines: usize,
    pub max_characters: usize,
    pub mode: Mode,
    pub update_view: bool,
    pub highlighted_text: HighlightedText,
    pub gutter_info: Vec<GutterInfo>,
    pub relative_cursor: Cursor,
    pub buffer_idx: Option<u32>,
    pub clipboard_ctx: ClipboardContext,
    pub diagnostics: HashMap<String, types::PublishDiagnostics>,
    pub modal: Modal,
    pub diagnostics_overlay: DiagnosticsOverlay,
    pub info_modal: InfoModal,
    pub completion_menu: CompletionMenu,
    pub signature_information: SignatureInformation,
    pub keybind_handler: KeybindHandler,
    pub ai_state: AIState,
}

impl EditorState {
    pub fn new(rt: tokio::runtime::Runtime) -> Self {
        let (sender, receiver) = mpsc::channel::<AsyncResult>(32);
        let (file_event_sender, file_event_receiver) = mpsc::channel::<NotifyResult<Event>>(32);

        let rt_handle = rt.handle().clone();
        let watcher = RecommendedWatcher::new(
            move |res| {
                rt_handle.block_on(async {
                    file_event_sender.clone().send(res).await.unwrap();
                });
            },
            Config::default(),
        )
        .expect("Failed to create file watcher");

        let initial_folder = std::path::absolute("/")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        Self {
            quit: false,
            rt,
            async_handle: AsyncHandle { sender, receiver },
            file_event_receiver,
            file_watcher: watcher,
            preferences: Preferences::default(),
            buffers: HashMap::new(),
            next_id: 0,
            workspace_folder: initial_folder.clone(),
            current_folder: initial_folder,
            visible_lines: 0,
            max_characters: 0,
            mode: Mode::Normal,
            instances: HashMap::new(),
            highlighted_text: vec![],
            gutter_info: vec![],
            buffer_idx: None,
            modal: Modal::default(),
            relative_cursor: Cursor { row: 0, column: 0 },
            update_view: true,
            clipboard_ctx: ClipboardContext::new().unwrap(),
            diagnostics: HashMap::new(),
            diagnostics_overlay: DiagnosticsOverlay::default(),
            info_modal: InfoModal::default(),
            completion_menu: CompletionMenu::new(5),
            signature_information: SignatureInformation::default(),
            keybind_handler: KeybindHandler::default(),
            ai_state: AIState::default(),
        }
    }

    pub fn add_buffer(&mut self, buffer: LineBuffer) -> u32 {
        if let Some((idx, _)) = self
            .buffers
            .iter()
            .find(|(_, buf)| buf.file_path == buffer.file_path)
        {
            *idx
        } else {
            // Watch the file path if it exists before inserting the buffer
            if let Some(path_str) = &buffer.file_path {
                let path = Path::new(path_str);
                if path.exists() {
                    // Use expect for now, consider proper error handling later
                    self.file_watcher
                        .watch(path, RecursiveMode::NonRecursive)
                        .expect("Failed to watch file path");
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

    pub fn cycle_buffer(&mut self, reverse: bool) {
        if self.buffer_idx.is_some() {
            if reverse {
                self.buffer_idx = if self.buffer_idx.unwrap() == 0 {
                    Some((self.buffers.len() - 1).try_into().unwrap())
                } else {
                    Some(self.buffer_idx.unwrap() - 1)
                };
            } else {
                self.buffer_idx = if self.buffer_idx.unwrap() == self.buffers.len() as u32 - 1 {
                    Some(0)
                } else {
                    Some(self.buffer_idx.unwrap() + 1)
                };
            }
        }
    }

    pub fn get_buffer_by_id(&self, id: u32) -> (&LineBuffer, &BufferInstance) {
        (
            self.buffers.get(&id).unwrap(),
            self.instances.get(&id).unwrap(),
        )
    }

    pub fn get_buffer_by_id_mut(&mut self, id: u32) -> (&mut LineBuffer, &mut BufferInstance) {
        (
            self.buffers.get_mut(&id).unwrap(),
            self.instances.get_mut(&id).unwrap(),
        )
    }

    pub fn spawn_lsp(&self, language: Language) -> Option<LSPClientHandle> {
        let command: Option<(&str, &[&str])> = match language {
            Language::Rust => Some(("rust-analyzer", &[])),
            Language::Python => Some(("uv", &["run", "pylsp"])),
            _ => None,
        };
        if let Some(command) = command {
            if which::which(command.0).is_ok() {
                return Some(
                    self.rt
                        .block_on(async { start_lsp(command.0, command.1).await.unwrap() }),
                );
            } else {
                return None;
            }
        }
        None
    }
}

type ModalOnInput =
    fn(&String, state: &mut EditorState, lsp_handles: &mut HashMap<Language, LSPClientHandle>);

type ModalOnSelect = fn(
    String,
    &(String, String),
    bool,
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
);

pub struct Modal {
    pub open: bool,
    pub input: String,
    pub options: Vec<(String, String)>,
    pub selection: Option<usize>,
    pub on_input: Option<ModalOnInput>,
    pub on_select: Option<ModalOnSelect>,
}

impl Modal {
    pub fn new() -> Self {
        Self {
            open: false,
            input: String::new(),
            options: vec![],
            selection: None,
            on_input: None,
            on_select: None,
        }
    }

    pub fn clear(&mut self) {
        self.input = String::new();
        self.options = vec![];
        self.selection = None;
        self.on_input = None;
        self.on_select = None;
    }

    pub fn set_modal_on_input(&mut self, on_input: ModalOnInput) {
        self.on_input = Some(on_input);
    }

    pub fn set_modal_on_select(&mut self, on_select: ModalOnSelect) {
        self.on_select = Some(on_select);
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;

        if !self.options.is_empty() {
            self.selection = Some(0);
        } else {
            self.selection = None;
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.clear();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.clear();
    }

    pub fn select_next(&mut self) {
        if !self.options.is_empty() {
            if self.selection.is_none() {
                self.selection = Some(0);
            } else if self.selection.unwrap() < self.options.len() - 1 {
                self.selection = Some(self.selection.unwrap() + 1);
            } else {
                self.selection = Some(0);
            }
        }
    }

    pub fn select_prev(&mut self) {
        if !self.options.is_empty() {
            if self.selection.is_none() {
                self.selection = Some(self.options.len() - 1);
            } else if self.selection.unwrap() > 0 {
                self.selection = Some(self.selection.unwrap() - 1);
            } else {
                self.selection = Some(self.options.len() - 1);
            }
        }
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self::new()
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

pub struct InfoModal {
    pub active: bool,
    pub content: String,
}

impl InfoModal {
    pub fn new() -> Self {
        Self {
            active: false,
            content: String::new(),
        }
    }

    pub fn open(&mut self, content: String) {
        self.content = content;
        self.active = true;
    }

    pub fn close(&mut self) {
        self.active = false;
    }
}

impl Default for InfoModal {
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

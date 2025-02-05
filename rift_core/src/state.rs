use std::collections::HashMap;

use copypasta::ClipboardContext;
use tokio::sync::mpsc;

use crate::{
    buffer::{
        instance::{BufferInstance, Cursor, GutterInfo, Language},
        line_buffer::{HighlightedText, LineBuffer},
    },
    concurrent::{AsyncHandle, AsyncResult},
    io::file_io::FolderEntry,
    lsp::{
        client::{start_lsp, LSPClientHandle},
        types,
    },
    preferences::Preferences,
};

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

pub struct EditorState {
    pub rt: tokio::runtime::Runtime,
    pub async_handle: AsyncHandle,
    pub preferences: Preferences,
    pub buffers: HashMap<u32, LineBuffer>,
    pub instances: HashMap<u32, BufferInstance>,
    next_id: u32,
    pub workspace_folder: String,
    pub visible_lines: usize,
    pub max_characters: usize,
    pub mode: Mode,
    pub update_view: bool,
    pub highlighted_text: HighlightedText,
    pub gutter_info: Vec<GutterInfo>,
    pub relative_cursor: Cursor,
    pub buffer_idx: Option<u32>,
    pub modal_open: bool,
    pub modal_options: Vec<FolderEntry>,
    pub modal_options_filtered: Vec<FolderEntry>,
    pub modal_selection_idx: Option<usize>,
    pub modal_input: String,
    pub clipboard_ctx: ClipboardContext,
    pub diagnostics: HashMap<String, types::PublishDiagnostics>,
}

impl EditorState {
    pub fn new(rt: tokio::runtime::Runtime) -> Self {
        let (sender, receiver) = mpsc::channel::<AsyncResult>(32);
        Self {
            rt,
            async_handle: AsyncHandle { sender, receiver },
            preferences: Preferences::default(),
            buffers: HashMap::new(),
            next_id: 0,
            workspace_folder: std::path::absolute("/")
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
            visible_lines: 0,
            max_characters: 0,
            mode: Mode::Normal,
            instances: HashMap::new(),
            highlighted_text: vec![],
            gutter_info: vec![],
            buffer_idx: None,
            modal_open: false,
            modal_options: vec![],
            modal_options_filtered: vec![],
            modal_selection_idx: None,
            modal_input: "".to_string(),
            relative_cursor: Cursor { row: 0, column: 0 },
            update_view: true,
            clipboard_ctx: ClipboardContext::new().unwrap(),
            diagnostics: HashMap::new(),
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
            _ => None,
        };
        if let Some(command) = command {
            return Some(
                self.rt
                    .block_on(async { start_lsp(command.0, command.1).await.unwrap() }),
            );
        }
        None
    }
}

type ModalOnInput = fn(
    &String,
    &Vec<String>,
    state: &mut EditorState,
    lsp_handle: &mut Option<&mut LSPClientHandle>,
) -> Vec<String>;

type ModalOnSelect =
    fn(String, state: &mut EditorState, lsp_handle: &mut Option<&mut LSPClientHandle>);

pub struct Modal {
    pub open: bool,
    pub input: String,
    pub options: Vec<String>,
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

    pub fn set_input(
        &mut self,
        input: String,
        state: &mut EditorState,
        lsp_handle: &mut Option<&mut LSPClientHandle>,
    ) {
        self.input = input;
        if let Some(on_input) = self.on_input {
            self.options = on_input(&self.input, &self.options, state, lsp_handle);
        }

        if !self.options.is_empty() {
            self.selection = Some(0);
        } else {
            self.selection = None;
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.clear();
    }

    pub fn select(
        &mut self,
        state: &mut EditorState,
        lsp_handle: &mut Option<&mut LSPClientHandle>,
    ) {
        if let Some(on_select) = self.on_select {
            if let Some(selection) = self.selection {
                on_select(
                    self.options.get(selection).unwrap().to_string(),
                    state,
                    lsp_handle,
                );
            }
        }
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

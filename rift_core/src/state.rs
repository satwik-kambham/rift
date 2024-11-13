use std::collections::HashMap;

use crate::{
    buffer::{
        instance::{BufferInstance, Cursor, GutterInfo},
        line_buffer::{HighlightedText, LineBuffer},
    },
    io::file_io::FolderEntry,
};

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

pub struct EditorState {
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
}

impl EditorState {
    pub fn new() -> Self {
        Self {
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
        }
    }

    pub fn add_buffer(&mut self, buffer: LineBuffer) -> u32 {
        self.buffers.insert(self.next_id, buffer);
        self.instances
            .insert(self.next_id, BufferInstance::new(self.next_id));
        self.next_id += 1;
        self.next_id - 1
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
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

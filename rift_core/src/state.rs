use std::collections::HashMap;

use crate::buffer::{
    instance::{BufferInstance, GutterInfo},
    line_buffer::{HighlightedText, LineBuffer},
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
    pub visible_lines: usize,
    pub max_characters: usize,
    pub mode: Mode,
    pub highlighted_text: HighlightedText,
    pub gutter_info: Vec<GutterInfo>,
    pub buffer_idx: Option<u32>,
    pub modal_open: bool,
    pub modal_options: Vec<String>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            next_id: 0,
            visible_lines: 0,
            max_characters: 0,
            mode: Mode::Normal,
            instances: HashMap::new(),
            highlighted_text: vec![],
            gutter_info: vec![],
            buffer_idx: None,
            modal_open: false,
            modal_options: vec![],
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

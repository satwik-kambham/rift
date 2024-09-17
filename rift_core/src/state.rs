use std::collections::HashMap;

use crate::buffer::line_buffer::LineBuffer;

#[derive(Debug, Default)]
pub struct EditorState {
    pub buffers: HashMap<u32, LineBuffer>,
    next_id: u32,
    pub visible_lines: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            next_id: 0,
            visible_lines: 0,
        }
    }

    pub fn add_buffer(&mut self, buffer: LineBuffer) -> u32 {
        self.buffers.insert(self.next_id, buffer);
        self.next_id += 1;
        self.next_id - 1
    }

    pub fn remove_buffer(&mut self, id: u32) {
        self.buffers.remove(&id);
    }

    pub fn get_buffer_by_id(&self, id: u32) -> &LineBuffer {
        self.buffers.get(&id).unwrap()
    }

    pub fn get_buffer_by_id_mut(&mut self, id: u32) -> &mut LineBuffer {
        self.buffers.get_mut(&id).unwrap()
    }
}

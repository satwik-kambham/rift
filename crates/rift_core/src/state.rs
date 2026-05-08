use crate::compositor::Compositor;

pub struct EditorState {
    pub quit: bool,
    pub compositor: Compositor,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            quit: false,
            compositor: Compositor::new(),
        }
    }
}

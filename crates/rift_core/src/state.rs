use crate::compositor::Compositor;

pub struct EditorState {
    compositor: Compositor,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            compositor: Compositor::new(),
        }
    }
}

use rift_core::state::EditorState;

pub struct App {
    state: EditorState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
        }
    }

    pub fn run(&mut self) {}
}

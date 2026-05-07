pub struct RopeBufferInstance {}

impl RopeBufferInstance {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct RopeBuffer {
    buffer: ropey::Rope,
}

impl RopeBuffer {
    pub fn new(initial_text: &str) -> Self {
        let buffer = ropey::Rope::from_str(initial_text);
        Self { buffer }
    }
}

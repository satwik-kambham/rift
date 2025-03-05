// Dynamic keybind definition with keybind chaining
// Format:
// MODE Modifier-Key Key ...
//
// Case agnostic
// MODE - ALL, NOR, INS
// Modifier - C (Control), M (Alt), S (Shift)

pub struct KeybindHandler {
    cancel: String,
    running_sequence: String,
}

impl KeybindHandler {
    pub fn new() -> Self {
        Self {
            cancel: "Esc".to_string(),
            running_sequence: "".to_string(),
        }
    }
}

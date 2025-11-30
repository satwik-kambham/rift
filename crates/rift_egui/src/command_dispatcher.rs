use std::collections::{HashMap, HashSet};

use egui::Ui;
use rift_core::{
    actions::perform_action, buffer::instance::Language, lsp::client::LSPClientHandle,
    state::EditorState,
};

/// Util method that functions as ternary operator
fn upper<'a>(shift: bool, base: &'a str, modified: &'a str) -> &'a str {
    if shift { modified } else { base }
}

pub struct CommandDispatcher {}

impl CommandDispatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &self,
        ui: &mut Ui,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        ui.input(|i| {
            for event in &i.raw.events {
                state.update_view = true;
                if let egui::Event::Key {
                    key,
                    physical_key: _,
                    pressed,
                    repeat: _,
                    modifiers,
                } = event
                    && *pressed
                    && !(state.completion_menu.active
                        && matches!(key, egui::Key::Tab | egui::Key::Enter))
                {
                    let key = match key {
                        egui::Key::ArrowDown => "Down",
                        egui::Key::ArrowLeft => "Left",
                        egui::Key::ArrowRight => "Right",
                        egui::Key::ArrowUp => "Up",
                        egui::Key::Escape => "Escape",
                        egui::Key::Tab => "Tab",
                        egui::Key::Backspace => "Backspace",
                        egui::Key::Enter => "Enter",
                        egui::Key::Space => "Space",
                        egui::Key::Insert => "Insert",
                        egui::Key::Delete => "Delete",
                        egui::Key::Home => "Home",
                        egui::Key::End => "End",
                        egui::Key::PageUp => "PageUp",
                        egui::Key::PageDown => "PageDown",
                        egui::Key::Copy => "Copy",
                        egui::Key::Cut => "Cut",
                        egui::Key::Paste => "Paste",
                        egui::Key::Semicolon => ";",
                        egui::Key::Colon => ":",
                        egui::Key::Slash => "/",
                        egui::Key::Questionmark => "?",
                        egui::Key::Backslash => "\\",
                        egui::Key::Pipe => "|",
                        egui::Key::Plus => "+",
                        egui::Key::Equals => "=",
                        egui::Key::OpenBracket => "[",
                        egui::Key::CloseBracket => "]",
                        egui::Key::OpenCurlyBracket => "{",
                        egui::Key::CloseCurlyBracket => "}",
                        egui::Key::Backtick => upper(modifiers.shift, "`", "~"),
                        egui::Key::Minus => upper(modifiers.shift, "-", "_"),
                        egui::Key::Period => upper(modifiers.shift, ".", ">"),
                        egui::Key::Comma => upper(modifiers.shift, ",", "<"),
                        egui::Key::Quote => upper(modifiers.shift, "'", "\""),
                        egui::Key::Num1 => "1",
                        egui::Key::Exclamationmark => "!",
                        egui::Key::Num2 => upper(modifiers.shift, "2", "@"),
                        egui::Key::Num3 => upper(modifiers.shift, "3", "#"),
                        egui::Key::Num4 => upper(modifiers.shift, "4", "$"),
                        egui::Key::Num5 => upper(modifiers.shift, "5", "%"),
                        egui::Key::Num6 => upper(modifiers.shift, "6", "^"),
                        egui::Key::Num7 => upper(modifiers.shift, "7", "&"),
                        egui::Key::Num8 => upper(modifiers.shift, "8", "*"),
                        egui::Key::Num9 => upper(modifiers.shift, "9", "("),
                        egui::Key::Num0 => upper(modifiers.shift, "0", ")"),
                        egui::Key::A => upper(modifiers.shift, "a", "A"),
                        egui::Key::B => upper(modifiers.shift, "b", "B"),
                        egui::Key::C => upper(modifiers.shift, "c", "C"),
                        egui::Key::D => upper(modifiers.shift, "d", "D"),
                        egui::Key::E => upper(modifiers.shift, "e", "E"),
                        egui::Key::F => upper(modifiers.shift, "f", "F"),
                        egui::Key::G => upper(modifiers.shift, "g", "G"),
                        egui::Key::H => upper(modifiers.shift, "h", "H"),
                        egui::Key::I => upper(modifiers.shift, "i", "I"),
                        egui::Key::J => upper(modifiers.shift, "j", "J"),
                        egui::Key::K => upper(modifiers.shift, "k", "K"),
                        egui::Key::L => upper(modifiers.shift, "l", "L"),
                        egui::Key::M => upper(modifiers.shift, "m", "M"),
                        egui::Key::N => upper(modifiers.shift, "n", "N"),
                        egui::Key::O => upper(modifiers.shift, "o", "O"),
                        egui::Key::P => upper(modifiers.shift, "p", "P"),
                        egui::Key::Q => upper(modifiers.shift, "q", "Q"),
                        egui::Key::R => upper(modifiers.shift, "r", "R"),
                        egui::Key::S => upper(modifiers.shift, "s", "S"),
                        egui::Key::T => upper(modifiers.shift, "t", "T"),
                        egui::Key::U => upper(modifiers.shift, "u", "U"),
                        egui::Key::V => upper(modifiers.shift, "v", "V"),
                        egui::Key::W => upper(modifiers.shift, "w", "W"),
                        egui::Key::X => upper(modifiers.shift, "x", "X"),
                        egui::Key::Y => upper(modifiers.shift, "y", "Y"),
                        egui::Key::Z => upper(modifiers.shift, "z", "Z"),
                        egui::Key::F1 => "F1",
                        egui::Key::F2 => "F2",
                        egui::Key::F3 => "F3",
                        egui::Key::F4 => "F4",
                        egui::Key::F5 => "F5",
                        egui::Key::F6 => "F6",
                        egui::Key::F7 => "F7",
                        egui::Key::F8 => "F8",
                        egui::Key::F9 => "F9",
                        egui::Key::F10 => "F10",
                        egui::Key::F11 => "F11",
                        egui::Key::F12 => "F12",
                        _ => "",
                    };
                    let mut modifiers_set = HashSet::new();
                    if modifiers.alt {
                        modifiers_set.insert("m".to_string());
                    } else if modifiers.ctrl {
                        modifiers_set.insert("c".to_string());
                    } else if modifiers.shift {
                        modifiers_set.insert("s".to_string());
                    }

                    if let Some(action) = state.keybind_handler.handle_input(
                        state.buffer_idx,
                        state.is_active_buffer_special(),
                        state.mode.clone(),
                        key.to_string(),
                        modifiers_set,
                    ) {
                        perform_action(action, state, lsp_handles);
                    }
                }
            }
        });
    }
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

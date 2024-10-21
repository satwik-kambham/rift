use egui::Ui;
use rift_core::{
    buffer::line_buffer::LineBuffer,
    io::file_io,
    state::{EditorState, Mode},
};

pub struct CommandDispatcher {}

impl CommandDispatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&self, ui: &mut Ui, state: &mut EditorState) {
        ui.input(|i| {
            if !state.modal_open {
                for event in &i.raw.events {
                    state.update_view = true;
                    match event {
                        egui::Event::Text(text) => {
                            if matches!(state.mode, Mode::Insert) {
                                let (buffer, instance) =
                                    state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                let cursor = buffer.insert_text(text, &instance.cursor);
                                instance.cursor = cursor;
                                instance.selection.cursor = instance.cursor;
                                instance.selection.mark = instance.cursor;
                            }
                        }
                        egui::Event::Key {
                            key,
                            physical_key: _,
                            pressed,
                            repeat: _,
                            modifiers,
                        } => {
                            if *pressed {
                                match key {
                                    egui::Key::Escape => {
                                        state.mode = Mode::Normal;
                                    }
                                    egui::Key::I => {
                                        if matches!(state.mode, Mode::Normal) {
                                            state.mode = Mode::Insert;
                                            return;
                                        }
                                    }
                                    egui::Key::O => {
                                        if matches!(state.mode, Mode::Normal) {
                                            state.mode = Mode::Insert;
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            instance.cursor = instance.selection.cursor;
                                            let indent_size =
                                                buffer.get_indentation_level(instance.cursor.row);
                                            let cursor = buffer.insert_text("\n", &instance.cursor);
                                            instance.cursor = cursor;
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                            instance.selection = buffer
                                                .add_indentation(&instance.selection, indent_size);
                                            instance.cursor = instance.selection.cursor;
                                            return;
                                        }
                                    }
                                    egui::Key::Comma => {
                                        if matches!(state.mode, Mode::Normal) && modifiers.shift {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            instance.selection =
                                                buffer.remove_indentation(&instance.selection, 4);
                                            instance.cursor = instance.selection.cursor;
                                        }
                                    }
                                    egui::Key::Period => {
                                        if matches!(state.mode, Mode::Normal) && modifiers.shift {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            instance.selection =
                                                buffer.add_indentation(&instance.selection, 4);
                                            instance.cursor = instance.selection.cursor;
                                        }
                                    }
                                    egui::Key::X => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.selection.cursor;
                                            }
                                            instance.selection =
                                                buffer.select_line(&instance.selection);
                                            instance.cursor = instance.selection.cursor;
                                        }
                                    }
                                    egui::Key::W => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.selection.cursor;
                                            }
                                            instance.selection =
                                                buffer.select_word(&instance.selection);
                                            instance.cursor = instance.selection.cursor;
                                        }
                                    }
                                    egui::Key::F => {
                                        if matches!(state.mode, Mode::Normal) {
                                            state.modal_open = true;
                                            state.modal_options =
                                                file_io::get_directory_entries("/").unwrap();
                                            state.modal_options_filtered =
                                                state.modal_options.clone();
                                            state.modal_selection_idx = None;
                                            state.modal_input = "/".into();
                                            return;
                                        }
                                    }
                                    egui::Key::S => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, _instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.modified = false;
                                            file_io::override_file_content(
                                                &buffer.file_path.clone().unwrap(),
                                                buffer.get_content("\n".into()),
                                            )
                                            .unwrap();
                                        }
                                    }
                                    egui::Key::ArrowDown => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_down(
                                            &mut instance.cursor,
                                            instance.column_level,
                                        );
                                        instance.selection.cursor = instance.cursor;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::ArrowUp => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_up(
                                            &mut instance.cursor,
                                            instance.column_level,
                                        );
                                        instance.selection.cursor = instance.cursor;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::ArrowLeft => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_left(&mut instance.cursor);
                                        instance.selection.cursor = instance.cursor;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::ArrowRight => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_right(&mut instance.cursor);
                                        instance.selection.cursor = instance.cursor;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::J => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.move_cursor_down(
                                                &mut instance.cursor,
                                                instance.column_level,
                                            );
                                            instance.selection.cursor = instance.cursor;
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.cursor;
                                            }
                                        }
                                    }
                                    egui::Key::K => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.move_cursor_up(
                                                &mut instance.cursor,
                                                instance.column_level,
                                            );
                                            instance.selection.cursor = instance.cursor;
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.cursor;
                                            }
                                        }
                                    }
                                    egui::Key::H => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.move_cursor_left(&mut instance.cursor);
                                            instance.selection.cursor = instance.cursor;
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.cursor;
                                            }
                                        }
                                    }
                                    egui::Key::L => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.move_cursor_right(&mut instance.cursor);
                                            instance.selection.cursor = instance.cursor;
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.cursor;
                                            }
                                        }
                                    }
                                    egui::Key::Backspace => {
                                        if matches!(state.mode, Mode::Insert) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                            buffer.move_cursor_left(&mut instance.selection.mark);

                                            let (_text, cursor) =
                                                buffer.remove_text(&instance.selection);
                                            instance.cursor = cursor;
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::Enter => {
                                        if matches!(state.mode, Mode::Insert) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            let cursor = buffer.insert_text("\n", &instance.cursor);
                                            instance.cursor = cursor;
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                for event in &i.raw.events {
                    state.update_view = true;
                    match event {
                        egui::Event::Text(text) => {
                            state.modal_input.push_str(text);
                            state.modal_options_filtered = state
                                .modal_options
                                .iter()
                                .filter(|entry| entry.path.starts_with(&state.modal_input))
                                .cloned()
                                .collect();
                        }
                        egui::Event::Key {
                            key,
                            physical_key: _,
                            pressed,
                            repeat: _,
                            modifiers: _,
                        } => {
                            if *pressed {
                                match key {
                                    egui::Key::Tab => {
                                        if !state.modal_options_filtered.is_empty() {
                                            if state.modal_selection_idx.is_none() {
                                                state.modal_selection_idx = Some(0);
                                            } else {
                                                state.modal_selection_idx =
                                                    Some(state.modal_selection_idx.unwrap() + 1);
                                                if state.modal_selection_idx.unwrap()
                                                    >= state.modal_options_filtered.len()
                                                {
                                                    state.modal_selection_idx = Some(0);
                                                }
                                            }

                                            state.modal_input = state.modal_options_filtered
                                                [state.modal_selection_idx.unwrap()]
                                            .path
                                            .clone();
                                        } else {
                                            state.modal_selection_idx = None;
                                        }
                                    }
                                    egui::Key::Backspace => {
                                        state.modal_input.pop();
                                        state.modal_options_filtered = state
                                            .modal_options
                                            .iter()
                                            .filter(|entry| {
                                                entry.path.starts_with(&state.modal_input)
                                            })
                                            .cloned()
                                            .collect();
                                    }
                                    egui::Key::Enter => {
                                        if state.modal_selection_idx.is_some() {
                                            let entry = &state.modal_options_filtered
                                                [state.modal_selection_idx.unwrap()];
                                            if !entry.is_dir {
                                                let path = &entry.path;
                                                let initial_text =
                                                    file_io::read_file_content(path).unwrap();
                                                let buffer = LineBuffer::new(
                                                    initial_text,
                                                    Some(path.to_string()),
                                                );
                                                state.buffer_idx = Some(state.add_buffer(buffer));
                                                state.modal_open = false;
                                                state.modal_options = vec![];
                                                state.modal_options_filtered = vec![];
                                                state.modal_selection_idx = None;
                                                state.modal_input = "".into();
                                            } else {
                                                state.modal_input = entry.path.clone();

                                                #[cfg(target_os = "windows")]
                                                {
                                                    state.modal_input.push('\\');
                                                }

                                                #[cfg(any(
                                                    target_os = "linux",
                                                    target_os = "macos"
                                                ))]
                                                {
                                                    state.modal_input.push('/');
                                                }

                                                state.modal_options =
                                                    file_io::get_directory_entries(&entry.path)
                                                        .unwrap();
                                                state.modal_options_filtered =
                                                    state.modal_options.clone();
                                                state.modal_selection_idx = None;
                                            }
                                        }
                                    }
                                    egui::Key::Escape => {
                                        state.modal_open = false;
                                        state.modal_options = vec![];
                                        state.modal_options_filtered = vec![];
                                        state.modal_selection_idx = None;
                                        state.modal_input = "".into();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

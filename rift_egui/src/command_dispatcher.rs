use egui::Ui;
use rift_core::{buffer::line_buffer::LineBuffer, io::file_io, state::EditorState};

pub struct CommandDispatcher {}

impl CommandDispatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &self,
        ui: &mut Ui,
        state: &mut EditorState,
        visible_lines: usize,
        max_characters: usize,
    ) {
        ui.input(|i| {
            for event in &i.raw.events {
                match event {
                    egui::Event::Key {
                        key,
                        physical_key: _,
                        pressed,
                        repeat: _,
                        modifiers: _,
                    } => {
                        if *pressed {
                            match key {
                                egui::Key::F1 => {
                                    let path = "/home/satwik/Documents/test.rs";
                                    let initial_text = file_io::read_file_content(path).unwrap();
                                    let buffer =
                                        LineBuffer::new(initial_text, Some(path.to_string()));
                                    state.buffer_idx = Some(state.add_buffer(buffer));
                                    let (buffer, instance) =
                                        state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                    let (lines, relative_cursor, gutter_info) = buffer
                                        .get_visible_lines(
                                            &mut instance.scroll,
                                            &instance.cursor,
                                            visible_lines,
                                            max_characters,
                                            "\n".into(),
                                        );

                                    state.highlighted_text = lines;
                                }
                                egui::Key::ArrowDown => {
                                    let (buffer, instance) =
                                        state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                    buffer.move_cursor_down(
                                        &mut instance.cursor,
                                        instance.column_level,
                                    );
                                }
                                egui::Key::ArrowUp => {
                                    let (buffer, instance) =
                                        state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                    buffer.move_cursor_up(
                                        &mut instance.cursor,
                                        instance.column_level,
                                    );
                                }
                                egui::Key::ArrowLeft => {
                                    let (buffer, instance) =
                                        state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                    buffer.move_cursor_left(&mut instance.cursor);
                                }
                                egui::Key::ArrowRight => {
                                    let (buffer, instance) =
                                        state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                    buffer.move_cursor_right(&mut instance.cursor);
                                }
                                egui::Key::Backspace => {
                                    let (buffer, instance) =
                                        state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                    instance.selection.cursor = instance.cursor;
                                    instance.selection.mark = instance.cursor;
                                    buffer.move_cursor_left(&mut instance.selection.mark);

                                    let (_text, cursor) = buffer.remove_text(&instance.selection);
                                    instance.cursor = cursor;
                                }
                                egui::Key::Enter => {
                                    let (buffer, instance) =
                                        state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                    let cursor = buffer.insert_text("\n", &instance.cursor);
                                    instance.cursor = cursor;
                                }
                                _ => {}
                            }
                        }
                    }
                    egui::Event::Text(text) => {
                        let (buffer, instance) =
                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                        let cursor = buffer.insert_text(text, &instance.cursor);
                        instance.cursor = cursor;
                    }
                    _ => {}
                }
            }
        });
    }
}

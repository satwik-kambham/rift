use std::collections::HashMap;

use egui::RichText;
use rift_core::{
    buffer::instance::Language,
    lsp::client::LSPClientHandle,
    preferences::Color,
    state::{CompletionMenu, EditorState},
};

pub struct CompletionMenuWidget {
    pub selection_color: Color,
}

impl CompletionMenuWidget {
    pub fn new(selection_color: Color) -> Self {
        Self { selection_color }
    }

    pub fn show(
        &self,
        char_width: f32,
        char_height: f32,
        gutter_width: f32,
        visible_lines: usize,
        ctx: &egui::Context,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) -> bool {
        if state.completion_menu.active {
            let offset = egui::Pos2 {
                x: (state.relative_cursor.column as f32 * char_width)
                    + gutter_width
                    + char_width
                    + state.preferences.editor_padding,
                y: (state.relative_cursor.row as f32 * char_height)
                    + char_height
                    + state.preferences.editor_padding,
            };
            let pivot = if visible_lines - 7 < state.relative_cursor.row {
                egui::Align2::LEFT_BOTTOM
            } else {
                egui::Align2::LEFT_TOP
            };
            egui::Window::new("completion_menu")
                .movable(false)
                .interactable(true)
                .order(egui::Order::Tooltip)
                .fixed_pos(offset)
                .pivot(pivot)
                .collapsible(false)
                .title_bar(false)
                .auto_sized()
                .show(ctx, |ui| {
                    for (idx, item) in state
                        .completion_menu
                        .items
                        .get(
                            state.completion_menu.start
                                ..state.completion_menu.start + state.completion_menu.max_items,
                        )
                        .unwrap_or(&state.completion_menu.items[state.completion_menu.start..])
                        .iter()
                        .enumerate()
                    {
                        if state.completion_menu.selection.unwrap_or(usize::MAX)
                            == state.completion_menu.start + idx
                        {
                            ui.label(
                                RichText::new(item.label.clone())
                                    .background_color(self.selection_color),
                            );
                        } else {
                            ui.label(item.label.clone());
                        }
                    }
                    self.handle_input(ui, state, lsp_handles);
                });
            return false;
        }
        true
    }

    pub fn handle_input(
        &self,
        ui: &mut egui::Ui,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        ui.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::Key {
                    key,
                    physical_key: _,
                    pressed,
                    repeat: _,
                    modifiers: _,
                } = event
                {
                    if *pressed {
                        match key {
                            egui::Key::Escape => {
                                state.completion_menu.close();
                            }
                            egui::Key::Tab => {
                                state.completion_menu.select_next();
                            }
                            egui::Key::Enter => {
                                let completion_item = state.completion_menu.select();
                                CompletionMenu::on_select(completion_item, state, lsp_handles);
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }
}

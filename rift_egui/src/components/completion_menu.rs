use egui::RichText;
use rift_core::{
    lsp::{client::LSPClientHandle, types},
    preferences::{Color, Preferences},
    state::EditorState,
};

pub struct CompletionMenu {
    pub items: Vec<types::CompletionItem>,
    pub active: bool,
    pub max_items: usize,
    pub start: usize,
    pub idx: usize,
    pub selection_color: Color,
}

impl CompletionMenu {
    pub fn new(max_items: usize, selection_color: Color) -> Self {
        Self {
            items: vec![],
            active: false,
            max_items,
            idx: 0,
            start: 0,
            selection_color,
        }
    }

    pub fn set_items(&mut self, items: Vec<types::CompletionItem>) {
        self.items = items;
        self.idx = 0;
        self.start = 0;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        state: &mut EditorState,
        preferences: &mut Preferences,
        lsp_handle: &mut LSPClientHandle,
    ) -> bool {
        if self.active {
            egui::Window::new("completion_menu")
                .movable(false)
                .interactable(true)
                .order(egui::Order::Tooltip)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .title_bar(false)
                .auto_sized()
                .show(ctx, |ui| {
                    for (idx, item) in self
                        .items
                        .get(self.start..self.start + self.max_items)
                        .unwrap_or(&self.items[self.start..])
                        .iter()
                        .enumerate()
                    {
                        if self.idx == self.start + idx {
                            ui.label(
                                RichText::new(item.label.clone())
                                    .background_color(self.selection_color),
                            );
                        } else {
                            ui.label(item.label.clone());
                        }
                    }
                    self.handle_input(ui, state, preferences, lsp_handle);
                });
            return false;
        }
        true
    }

    pub fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut EditorState,
        _preferences: &mut Preferences,
        lsp_handle: &mut LSPClientHandle,
    ) {
        ui.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::Key {
                    key,
                    physical_key: _,
                    pressed: _,
                    repeat: _,
                    modifiers: _,
                } = event
                {
                    match key {
                        egui::Key::Escape => {
                            self.active = false;
                        }
                        egui::Key::Tab => {
                            self.idx += 1;
                            if self.idx >= self.items.len() {
                                self.idx = 0;
                                self.start = 0;
                            }

                            if self.idx >= self.start + self.max_items {
                                self.start = self.idx;
                            }
                        }
                        egui::Key::Enter => {
                            let (buffer, instance) =
                                state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                            println!("{:#?}", self.items[self.idx]);
                            let _ = buffer.remove_text(
                                &self.items[self.idx].edit.range,
                                lsp_handle,
                                true,
                            );
                            let cursor = buffer.insert_text(
                                &self.items[self.idx].edit.text,
                                &self.items[self.idx].edit.range.mark,
                                lsp_handle,
                                true,
                            );
                            instance.cursor = cursor;
                            instance.selection.cursor = instance.cursor;
                            instance.selection.mark = instance.cursor;
                            instance.column_level = instance.cursor.column;
                            self.active = false;
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

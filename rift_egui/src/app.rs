use egui::{text::LayoutJob, Color32, FontId, Label, Rect, RichText};
use rift_core::state::EditorState;

use crate::command_dispatcher::CommandDispatcher;

pub struct App {
    dispatcher: CommandDispatcher,
    state: EditorState,
}

impl App {
    pub fn new() -> Self {
        Self {
            dispatcher: CommandDispatcher::new(),
            state: EditorState::default(),
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        let mut char_height = 0.0;
        let mut char_width = 0.0;
        egui::CentralPanel::default().show(ctx, |ui| {
            let label_response = ui.label(RichText::new("x").font(FontId::monospace(24.0)));
            char_width = label_response.rect.width();
            char_height = label_response.rect.height();
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let visible_lines = ((rect.height() / char_height).floor() as usize).saturating_sub(4);
            let max_characters = (rect.width() / char_width).floor() as usize;
            if visible_lines != self.state.visible_lines
                || max_characters != self.state.max_characters
            {
                self.state.visible_lines = visible_lines;
                self.state.max_characters = max_characters;
            }
            self.update_visible_lines(visible_lines, max_characters);

            for line in &self.state.highlighted_text {
                let mut job = LayoutJob::default();
                for token in line {
                    job.append(
                        &token.0,
                        0.0,
                        egui::TextFormat {
                            font_id: FontId::monospace(24.0),
                            color: Color32::LIGHT_RED,
                            ..Default::default()
                        },
                    )
                }
                ui.label(job);
            }

            self.dispatcher
                .show(ui, &mut self.state, visible_lines, max_characters);
        });
    }

    pub fn update_visible_lines(&mut self, visible_lines: usize, max_characters: usize) {
        if self.state.buffer_idx.is_some() {
            let (buffer, instance) = self
                .state
                .get_buffer_by_id_mut(self.state.buffer_idx.unwrap());
            let (lines, _relative_cursor, _gutter_info) = buffer.get_visible_lines(
                &mut instance.scroll,
                &instance.cursor,
                visible_lines,
                max_characters,
                "\n".into(),
            );
            self.state.highlighted_text = lines;
        }
    }
}

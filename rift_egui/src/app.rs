use egui::{text::LayoutJob, Color32, FontId, Label, Rect, RichText};
use rift_core::{buffer::instance::HighlightType, state::EditorState};

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
        let mut relative_cursor = rift_core::buffer::instance::Cursor { row: 0, column: 0 };
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::PLACEHOLDER,
                ..Default::default()
            })
            .show(ctx, |ui| {
                let label_response = ui.label(RichText::new("x").font(FontId::monospace(24.0)));
                char_width = label_response.rect.width();
                char_height = label_response.rect.height();
            });
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::from_rgb(33, 37, 43),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                let rect = ui.max_rect();
                let visible_lines = (rect.height() / char_height).floor() as usize;
                let max_characters = (rect.width() / char_width).floor() as usize;
                if visible_lines != self.state.visible_lines
                    || max_characters != self.state.max_characters
                {
                    self.state.visible_lines = visible_lines;
                    self.state.max_characters = max_characters;
                }
                relative_cursor = self.update_visible_lines(visible_lines, max_characters);

                for line in &self.state.highlighted_text {
                    let mut job = LayoutJob::default();
                    for token in line {
                        job.append(
                            &token.0,
                            0.0,
                            egui::TextFormat {
                                font_id: FontId::monospace(24.0),
                                color: match &token.1 {
                                    HighlightType::None => Color32::from_rgb(171, 178, 191),
                                    HighlightType::White => Color32::from_rgb(171, 178, 191),
                                    HighlightType::Red => Color32::from_rgb(224, 108, 117),
                                    HighlightType::Orange => Color32::from_rgb(209, 154, 102),
                                    HighlightType::Blue => Color32::from_rgb(95, 170, 232),
                                    HighlightType::Green => Color32::from_rgb(152, 195, 121),
                                    HighlightType::Purple => Color32::from_rgb(198, 120, 221),
                                    HighlightType::Yellow => Color32::from_rgb(229, 192, 123),
                                    HighlightType::Gray => Color32::from_rgb(92, 99, 112),
                                    HighlightType::Turquoise => Color32::from_rgb(86, 182, 194),
                                },
                                ..Default::default()
                            },
                        )
                    }
                    ui.label(job);
                }

                self.dispatcher
                    .show(ui, &mut self.state, visible_lines, max_characters);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::TRANSPARENT,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.put(
                    Rect::from_two_pos(
                        egui::Pos2 {
                            x: relative_cursor.column as f32 * char_width,
                            y: relative_cursor.row as f32 * char_height,
                        },
                        egui::Pos2 {
                            x: (relative_cursor.column as f32 * char_width) + char_width,
                            y: (relative_cursor.row as f32 * char_height) + char_height,
                        },
                    ),
                    Label::new(
                        RichText::new(" ")
                            .font(FontId::monospace(24.0))
                            .background_color(Color32::WHITE.gamma_multiply(0.8)),
                    ),
                );
            });
    }

    pub fn update_visible_lines(
        &mut self,
        visible_lines: usize,
        max_characters: usize,
    ) -> rift_core::buffer::instance::Cursor {
        if self.state.buffer_idx.is_some() {
            let (buffer, instance) = self
                .state
                .get_buffer_by_id_mut(self.state.buffer_idx.unwrap());
            let (lines, relative_cursor, _gutter_info) = buffer.get_visible_lines(
                &mut instance.scroll,
                &instance.cursor,
                visible_lines,
                max_characters,
                "\n".into(),
            );
            self.state.highlighted_text = lines;
            return relative_cursor;
        }
        rift_core::buffer::instance::Cursor { row: 0, column: 0 }
    }
}

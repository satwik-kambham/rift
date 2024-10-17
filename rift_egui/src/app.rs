use egui::{text::LayoutJob, Color32, FontId, Label, Rect, RichText};
use rift_core::{
    buffer::instance::HighlightType,
    state::{EditorState, Mode},
};

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
        let mut gutter_width = 0.0;
        egui::TopBottomPanel::bottom("status_line").show(ctx, |ui| {
            if self.state.buffer_idx.is_some() {
                let (_buffer, instance) =
                    self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
                ui.columns(2, |columns| {
                    let mode = &self.state.mode;
                    match mode {
                        Mode::Normal => columns[0].label(
                            RichText::new("NORMAL")
                                .color(Color32::from_rgb(24, 24, 24))
                                .background_color(Color32::from_rgb(203, 166, 247)),
                        ),
                        Mode::Insert => columns[0].label(
                            RichText::new("INSERT")
                                .color(Color32::from_rgb(24, 24, 24))
                                .background_color(Color32::from_rgb(166, 227, 161)),
                        ),
                    };
                    columns[1].label(format!(
                        "{}:{}",
                        instance.cursor.row + 1,
                        instance.cursor.column + 1
                    ));
                });
            }
        });
        egui::SidePanel::left("gutter")
            .frame(egui::Frame {
                fill: Color32::from_rgb(24, 24, 24),
                inner_margin: egui::Margin::same(8.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                let rect = ui.max_rect();
                gutter_width = rect.width() + 16.0;
                for gutter_line in &self.state.gutter_info {
                    let gutter_value = if gutter_line.wrapped {
                        ".".to_string()
                    } else {
                        format!("{}", gutter_line.start.row + 1)
                    };
                    ui.label(
                        RichText::new(gutter_value)
                            .font(FontId::monospace(24.0))
                            .color(Color32::from_rgb(92, 99, 112)),
                    );
                }
            });
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::from_rgb(24, 24, 24),
                inner_margin: egui::Margin::same(8.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let label_response = ui.label(RichText::new("x").font(FontId::monospace(24.0)));
                char_width = label_response.rect.width();
                char_height = label_response.rect.height();
            });
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::from_rgb(24, 24, 24),
                outer_margin: egui::Margin::same(8.0),
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
                                background: match &token.2 {
                                    true => Color32::from_rgb(108, 112, 134),
                                    false => Color32::TRANSPARENT,
                                },
                                ..Default::default()
                            },
                        )
                    }
                    ui.label(job);
                }

                self.dispatcher.show(ui, &mut self.state);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::TRANSPARENT,
                outer_margin: egui::Margin::same(8.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.put(
                    Rect::from_two_pos(
                        egui::Pos2 {
                            x: (relative_cursor.column as f32 * char_width) + gutter_width + 8.0,
                            y: (relative_cursor.row as f32 * char_height) + 8.0,
                        },
                        egui::Pos2 {
                            x: (relative_cursor.column as f32 * char_width)
                                + gutter_width
                                + char_width
                                + 8.0,
                            y: (relative_cursor.row as f32 * char_height) + char_height + 8.0,
                        },
                    ),
                    Label::new(
                        RichText::new(" ")
                            .font(FontId::monospace(24.0))
                            .background_color(Color32::from_rgb(166, 227, 161).gamma_multiply(0.8)),
                    ),
                );
            });
        if self.state.modal_open {
            egui::Window::new("modal")
                .movable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .show(ctx, |ui| {
                    ui.label(&self.state.modal_input);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (idx, entry) in self.state.modal_options.iter().enumerate() {
                            ui.label(RichText::new(&entry.name).color(
                                if self.state.modal_selection_idx.is_some()
                                    && idx == self.state.modal_selection_idx.unwrap()
                                {
                                    Color32::WHITE
                                } else {
                                    Color32::GRAY
                                },
                            ));
                        }
                    });
                });
        }
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
            let (lines, relative_cursor, gutter_info) = buffer.get_visible_lines(
                &mut instance.scroll,
                &instance.selection,
                visible_lines,
                max_characters,
                "\n".into(),
            );
            self.state.highlighted_text = lines;
            self.state.gutter_info = gutter_info;
            return relative_cursor;
        }
        rift_core::buffer::instance::Cursor { row: 0, column: 0 }
    }
}

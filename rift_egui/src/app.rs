use egui::{text::LayoutJob, Color32, FontId, Label, Rect, RichText};
use rift_core::{
    buffer::instance::HighlightType,
    preferences::Preferences,
    state::{EditorState, Mode},
};

use crate::command_dispatcher::CommandDispatcher;

pub struct App {
    dispatcher: CommandDispatcher,
    state: EditorState,
    preferences: Preferences,
}

impl App {
    pub fn new() -> Self {
        Self {
            dispatcher: CommandDispatcher::new(),
            state: EditorState::default(),
            preferences: Preferences::default(),
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        let mut char_height = 0.0;
        let mut char_width = 0.0;
        let mut gutter_width = 0.0;
        egui::TopBottomPanel::bottom("status_line")
            .frame(egui::Frame {
                fill: self.preferences.theme.status_bar_bg.into(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                if self.state.buffer_idx.is_some() {
                    let (buffer, instance) =
                        self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
                    ui.columns(3, |columns| {
                        let mode = &self.state.mode;
                        match mode {
                            Mode::Normal => columns[0].label(
                                RichText::new("NORMAL")
                                    .color(self.preferences.theme.status_bar_normal_mode_fg)
                                    .background_color(
                                        self.preferences.theme.status_bar_normal_mode_bg,
                                    ),
                            ),
                            Mode::Insert => columns[0].label(
                                RichText::new("INSERT")
                                    .color(self.preferences.theme.status_bar_insert_mode_fg)
                                    .background_color(
                                        self.preferences.theme.status_bar_insert_mode_bg,
                                    ),
                            ),
                        };
                        columns[1].label(format!(
                            "{}:{}",
                            instance.cursor.row + 1,
                            instance.cursor.column + 1
                        ));
                        columns[2].label(if buffer.modified { "U" } else { "" });
                    });
                }
            });
        egui::SidePanel::left("gutter")
            .frame(egui::Frame {
                fill: self.preferences.theme.gutter_bg.into(),
                inner_margin: egui::Margin::same(self.preferences.gutter_padding),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                let rect = ui.max_rect();
                gutter_width = rect.width() + self.preferences.gutter_padding * 2.0;
                for (idx, gutter_line) in self.state.gutter_info.iter().enumerate() {
                    let gutter_value = if gutter_line.wrapped {
                        ".".to_string()
                    } else {
                        format!("{}", gutter_line.start.row + 1)
                    };
                    if idx == self.state.relative_cursor.row {
                        ui.label(
                            RichText::new(gutter_value)
                                .font(FontId::monospace(self.preferences.editor_font_size as f32))
                                .color(self.preferences.theme.gutter_text_current_line),
                        );
                    } else {
                        ui.label(
                            RichText::new(gutter_value)
                                .font(FontId::monospace(self.preferences.editor_font_size as f32))
                                .color(self.preferences.theme.gutter_text),
                        );
                    }
                }
            });
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: self.preferences.theme.editor_bg.into(),
                inner_margin: egui::Margin::same(self.preferences.editor_padding),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let label_response = ui.label(
                    RichText::new("x")
                        .font(FontId::monospace(self.preferences.editor_font_size as f32)),
                );
                char_width = label_response.rect.width();
                char_height = label_response.rect.height();
            });
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: self.preferences.theme.editor_bg.into(),
                outer_margin: egui::Margin::same(self.preferences.editor_padding),
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
                    self.state.update_view = true;
                }

                if self.state.update_view {
                    self.state.relative_cursor =
                        self.update_visible_lines(visible_lines, max_characters);
                    self.state.update_view = false;
                }

                for line in &self.state.highlighted_text {
                    let mut job = LayoutJob::default();
                    for token in line {
                        job.append(
                            &token.0,
                            0.0,
                            egui::TextFormat {
                                font_id: FontId::monospace(
                                    self.preferences.editor_font_size as f32,
                                ),
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
                outer_margin: egui::Margin::same(self.preferences.editor_padding),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.put(
                    Rect::from_two_pos(
                        egui::Pos2 {
                            x: (self.state.relative_cursor.column as f32 * char_width)
                                + gutter_width
                                + self.preferences.editor_padding,
                            y: (self.state.relative_cursor.row as f32 * char_height)
                                + self.preferences.editor_padding,
                        },
                        egui::Pos2 {
                            x: (self.state.relative_cursor.column as f32 * char_width)
                                + gutter_width
                                + char_width
                                + self.preferences.editor_padding,
                            y: (self.state.relative_cursor.row as f32 * char_height)
                                + char_height
                                + self.preferences.editor_padding,
                        },
                    ),
                    Label::new(
                        RichText::new(" ")
                            .font(FontId::monospace(self.preferences.editor_font_size as f32))
                            .background_color(if matches!(self.state.mode, Mode::Normal) {
                                self.preferences.theme.cursor_normal_mode_bg
                            } else {
                                self.preferences.theme.cursor_insert_mode_bg
                            }),
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
                        for (idx, entry) in self.state.modal_options_filtered.iter().enumerate() {
                            ui.label(
                                RichText::new(&entry.name)
                                    .color(
                                        if self.state.modal_selection_idx.is_some()
                                            && idx == self.state.modal_selection_idx.unwrap()
                                        {
                                            Color32::WHITE
                                        } else {
                                            Color32::GRAY
                                        },
                                    )
                                    .size(self.preferences.ui_font_size as f32),
                            );
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

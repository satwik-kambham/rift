use std::time::Duration;

use egui::{
    FontDefinitions,
    FontFamily::{Monospace, Proportional},
    FontId, RichText,
    style::TextStyle,
    text::LayoutJob,
};
use rift_core::{
    actions::{Action, perform_action},
    buffer::instance::{Attribute, HighlightType},
    io::file_io::handle_file_event,
    lsp::{handle_lsp_messages, types},
    rendering::update_visible_lines,
    state::{EditorState, Mode},
};

use crate::{
    command_dispatcher::CommandDispatcher,
    components::{
        completion_menu::{CompletionMenuPosition, CompletionMenuWidget},
        diagnostics_overlay::show_diagnostics_overlay,
        menu_bar::show_menu_bar,
        signature_information::show_signature_information,
        status_line::show_status_line,
    },
    fonts::load_fonts,
};

pub struct App {
    dispatcher: CommandDispatcher,
    state: EditorState,
    font_definitions: FontDefinitions,
    completion_menu: CompletionMenuWidget,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut state = EditorState::new();
        state.post_initialization();

        let font_definitions = load_fonts(&mut state);

        Self {
            dispatcher: CommandDispatcher::default(),
            completion_menu: CompletionMenuWidget::new(),
            state,
            font_definitions,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        egui_extras::install_image_loaders(ctx);
        // Quit command
        if self.state.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Repaint every second
        ctx.request_repaint_after_secs(1.0);

        // Set fonts and global style
        ctx.set_fonts(self.font_definitions.clone());
        ctx.set_theme(egui::Theme::Dark);
        ctx.style_mut(|style| {
            style.text_styles = [
                (
                    TextStyle::Heading,
                    FontId::new(
                        self.state.preferences.ui_font_size_heading as f32,
                        Proportional,
                    ),
                ),
                (
                    TextStyle::Body,
                    FontId::new(self.state.preferences.ui_font_size as f32, Proportional),
                ),
                (
                    TextStyle::Monospace,
                    FontId::new(self.state.preferences.editor_font_size as f32, Monospace),
                ),
                (
                    TextStyle::Button,
                    FontId::new(
                        self.state.preferences.ui_font_size_button as f32,
                        Proportional,
                    ),
                ),
                (
                    TextStyle::Small,
                    FontId::new(
                        self.state.preferences.ui_font_size_small as f32,
                        Proportional,
                    ),
                ),
            ]
            .into();
            style.visuals.override_text_color = Some(self.state.preferences.theme.ui_text.into());
            style.visuals.widgets = egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    corner_radius: egui::CornerRadius::same(2),
                    fg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    corner_radius: egui::CornerRadius::same(2),
                    fg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    corner_radius: egui::CornerRadius::same(3),
                    fg_stroke: egui::Stroke::new(1.5, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    corner_radius: egui::CornerRadius::same(2),
                    fg_stroke: egui::Stroke::new(2.0, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    corner_radius: egui::CornerRadius::same(2),
                    fg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
            };
        });

        let mut viewport_rows = 0;
        let mut viewport_columns = 0;
        let show_gutter = !matches!(self.state.is_active_buffer_special(), Some(true));

        show_menu_bar(ctx, &mut self.state);
        let (char_width, char_height) = show_status_line(ctx, &mut self.state);

        if show_gutter {
            egui::SidePanel::left("gutter")
                .resizable(true)
                .show_separator_line(false)
                .min_width(60.0)
                .frame(egui::Frame {
                    fill: self.state.preferences.theme.gutter_bg.into(),
                    inner_margin: egui::Margin::same(self.state.preferences.gutter_padding),
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    for (idx, gutter_line) in self.state.gutter_info.iter().enumerate() {
                        let gutter_value = if gutter_line.wrapped {
                            ".".to_string()
                        } else {
                            format!("{}", gutter_line.start.row + 1)
                        };
                        if idx == self.state.relative_cursor.row {
                            ui.label(
                                RichText::new(gutter_value)
                                    .font(FontId::monospace(
                                        self.state.preferences.editor_font_size as f32,
                                    ))
                                    .color(self.state.preferences.theme.gutter_text_current_line),
                            );
                        } else {
                            ui.label(
                                RichText::new(gutter_value)
                                    .font(FontId::monospace(
                                        self.state.preferences.editor_font_size as f32,
                                    ))
                                    .color(self.state.preferences.theme.gutter_text),
                            );
                        }
                    }
                });
        }

        // Render editor
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: self.state.preferences.theme.editor_bg.into(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                let rect = ui.max_rect();
                let top_left = rect.left_top();
                viewport_rows = (rect.height() / char_height).floor() as usize;
                viewport_columns = (rect.width() / char_width).floor() as usize;

                // Run async callbacks
                if let Ok(async_result) = self.state.async_handle.receiver.try_recv() {
                    (async_result.callback)(async_result.result, &mut self.state);
                    self.state.update_view = true;
                }

                while let Ok(action_request) = self.state.event_reciever.try_recv() {
                    let result =
                        perform_action(action_request.action, &mut self.state).unwrap_or_default();
                    action_request.response_tx.send(result).unwrap();
                    self.state.update_view = true;
                    std::thread::sleep(Duration::from_millis(10));
                }

                // Handle file watcher events
                if let Ok(file_event_result) = self.state.file_event_receiver.try_recv() {
                    handle_file_event(file_event_result, &mut self.state);
                    self.state.update_view = true;
                }

                // Handle lsp messages
                handle_lsp_messages(&mut self.state);

                // Update on resize
                if self
                    .state
                    .set_viewport_size(viewport_rows, viewport_columns)
                {
                    self.state.update_view = true;
                    if self.state.init_rsl_complete {
                        let _ = perform_action(
                            Action::RunSource(format!(
                                "onViewportSizeChanged({}, {})",
                                viewport_rows, viewport_columns
                            )),
                            &mut self.state,
                        );
                    }
                }

                // Update rendered lines
                if self.state.update_view {
                    self.state.relative_cursor =
                        update_visible_lines(&mut self.state, viewport_rows, viewport_columns);
                    self.state.update_view = false;
                }

                // Render buffer
                for line in &self.state.highlighted_text {
                    let mut job = LayoutJob::default();
                    job.wrap.max_width = f32::INFINITY;
                    job.wrap.max_rows = 1;
                    // job.wrap.break_anywhere = true;
                    for token in line {
                        let mut format = egui::TextFormat {
                            font_id: FontId::monospace(
                                self.state.preferences.editor_font_size as f32,
                            ),
                            ..Default::default()
                        };
                        let mut attributes: Vec<&Attribute> = token.1.iter().collect();
                        attributes.sort();
                        for attribute in attributes {
                            match attribute {
                                Attribute::None => {}
                                Attribute::Visible => {}
                                Attribute::Underline => {}
                                Attribute::Highlight(highlight_type) => {
                                    format.color = match highlight_type {
                                        HighlightType::None => {
                                            self.state.preferences.theme.highlight_none.into()
                                        }
                                        HighlightType::White => {
                                            self.state.preferences.theme.highlight_white.into()
                                        }
                                        HighlightType::Red => {
                                            self.state.preferences.theme.highlight_red.into()
                                        }
                                        HighlightType::Orange => {
                                            self.state.preferences.theme.highlight_orange.into()
                                        }
                                        HighlightType::Blue => {
                                            self.state.preferences.theme.highlight_blue.into()
                                        }
                                        HighlightType::Green => {
                                            self.state.preferences.theme.highlight_green.into()
                                        }
                                        HighlightType::Purple => {
                                            self.state.preferences.theme.highlight_purple.into()
                                        }
                                        HighlightType::Yellow => {
                                            self.state.preferences.theme.highlight_yellow.into()
                                        }
                                        HighlightType::Gray => {
                                            self.state.preferences.theme.highlight_gray.into()
                                        }
                                        HighlightType::Turquoise => {
                                            self.state.preferences.theme.highlight_turquoise.into()
                                        }
                                    };
                                }
                                Attribute::Select => {
                                    format.background =
                                        self.state.preferences.theme.selection_bg.into();
                                }
                                Attribute::Cursor => {
                                    format.color = if matches!(self.state.mode, Mode::Normal) {
                                        self.state.preferences.theme.cursor_normal_mode_fg.into()
                                    } else {
                                        self.state.preferences.theme.cursor_insert_mode_fg.into()
                                    };
                                    format.background = if matches!(self.state.mode, Mode::Normal) {
                                        self.state.preferences.theme.cursor_normal_mode_bg.into()
                                    } else {
                                        self.state.preferences.theme.cursor_insert_mode_bg.into()
                                    };
                                }
                                Attribute::DiagnosticSeverity(severity) => {
                                    format.underline = egui::Stroke::new(
                                        1.0,
                                        match severity {
                                            types::DiagnosticSeverity::Error => {
                                                self.state.preferences.theme.error
                                            }
                                            types::DiagnosticSeverity::Warning => {
                                                self.state.preferences.theme.warning
                                            }
                                            types::DiagnosticSeverity::Information => {
                                                self.state.preferences.theme.information
                                            }
                                            types::DiagnosticSeverity::Hint => {
                                                self.state.preferences.theme.hint
                                            }
                                        },
                                    );
                                }
                            }
                        }
                        job.append(&token.0, 0.0, format);
                    }
                    ui.label(job);
                }

                // Handle keyboard events
                if !ctx.wants_keyboard_input() {
                    self.dispatcher.show(ui, &mut self.state);
                }

                let completion_position = CompletionMenuPosition {
                    char_width,
                    char_height,
                    top_left,
                    viewport_rows,
                };
                self.completion_menu
                    .show(completion_position, ctx, &mut self.state);

                if self.state.signature_information.should_render()
                    && self.state.relative_cursor.row > 1
                {
                    show_signature_information(char_width, char_height, top_left, ctx, &self.state);
                }
            });

        if self.state.diagnostics_overlay.should_render() {
            show_diagnostics_overlay(ctx, &self.state);
        }
    }
}

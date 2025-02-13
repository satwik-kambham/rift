use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
};

use egui::{
    text::LayoutJob, Color32, FontData, FontDefinitions, FontId, FontTweak, Label, Rect, RichText,
};
use rift_core::{
    actions::{perform_action, Action},
    buffer::instance::{Attribute, Cursor, HighlightType, Language, Range, Selection},
    lsp::{client::LSPClientHandle, types},
    state::{EditorState, Mode},
};

use crate::{
    command_dispatcher::CommandDispatcher,
    components::{
        completion_menu::CompletionMenu, diagnostics_overlay::DiagnosticsOverlay,
        info_modal::InfoModal,
    },
};

pub struct App {
    dispatcher: CommandDispatcher,
    state: EditorState,
    font_definitions: FontDefinitions,
    lsp_handles: HashMap<Language, LSPClientHandle>,
    info_modal: InfoModal,
    completion_menu: CompletionMenu,
    diagnostics_overlay: DiagnosticsOverlay,
    editor_focused: bool,
}

impl App {
    pub fn new(rt: tokio::runtime::Runtime) -> Self {
        let state = EditorState::new(rt);
        let mut fonts = FontDefinitions::default();
        let editor_font = font_kit::source::SystemSource::new()
            .select_best_match(
                &[font_kit::family_name::FamilyName::Title(
                    state.preferences.editor_font_family.to_owned(),
                )],
                &font_kit::properties::Properties::new(),
            )
            .unwrap();
        let ui_font = font_kit::source::SystemSource::new()
            .select_best_match(
                &[font_kit::family_name::FamilyName::Title(
                    state.preferences.ui_font_family.to_owned(),
                )],
                &font_kit::properties::Properties::new(),
            )
            .unwrap();
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, state.preferences.editor_font_family.to_owned());
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, state.preferences.ui_font_family.to_owned());
        match editor_font {
            font_kit::handle::Handle::Path { path, font_index } => {
                let mut font_content = Vec::new();
                File::open(path)
                    .unwrap()
                    .read_to_end(&mut font_content)
                    .unwrap();
                fonts.font_data.insert(
                    state.preferences.editor_font_family.to_owned(),
                    FontData {
                        font: std::borrow::Cow::Owned(font_content),
                        index: font_index,
                        tweak: FontTweak::default(),
                    },
                );
            }
            font_kit::handle::Handle::Memory { bytes, font_index } => {
                fonts.font_data.insert(
                    state.preferences.editor_font_family.to_owned(),
                    FontData {
                        font: std::borrow::Cow::Owned((*bytes).clone()),
                        index: font_index,
                        tweak: FontTweak::default(),
                    },
                );
            }
        }
        match ui_font {
            font_kit::handle::Handle::Path { path, font_index } => {
                let mut font_content = Vec::new();
                File::open(path)
                    .unwrap()
                    .read_to_end(&mut font_content)
                    .unwrap();
                fonts.font_data.insert(
                    state.preferences.ui_font_family.to_owned(),
                    FontData {
                        font: std::borrow::Cow::Owned(font_content),
                        index: font_index,
                        tweak: FontTweak::default(),
                    },
                );
            }
            font_kit::handle::Handle::Memory { bytes, font_index } => {
                fonts.font_data.insert(
                    state.preferences.ui_font_family.to_owned(),
                    FontData {
                        font: std::borrow::Cow::Owned((*bytes).clone()),
                        index: font_index,
                        tweak: FontTweak::default(),
                    },
                );
            }
        }

        let lsp_handles = HashMap::new();

        Self {
            dispatcher: CommandDispatcher::default(),
            completion_menu: CompletionMenu::new(5, state.preferences.theme.selection_bg),
            state,
            font_definitions: fonts,
            lsp_handles,
            info_modal: InfoModal::default(),
            diagnostics_overlay: DiagnosticsOverlay::default(),
            editor_focused: true,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        ctx.set_fonts(self.font_definitions.clone());
        ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(self.state.preferences.theme.ui_text.into());
            style.visuals.widgets = egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    rounding: egui::Rounding::same(2.0),
                    fg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    rounding: egui::Rounding::same(2.0),
                    fg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    rounding: egui::Rounding::same(3.0),
                    fg_stroke: egui::Stroke::new(1.5, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    rounding: egui::Rounding::same(2.0),
                    fg_stroke: egui::Stroke::new(2.0, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: self.state.preferences.theme.ui_bg_fill.into(),
                    weak_bg_fill: self.state.preferences.theme.ui_weak_bg_fill.into(),
                    bg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_bg_stroke),
                    rounding: egui::Rounding::same(2.0),
                    fg_stroke: egui::Stroke::new(1.0, self.state.preferences.theme.ui_fg_stroke),
                    expansion: 0.0,
                },
            };
        });
        let mut char_height = 0.0;
        let mut char_width = 0.0;
        let mut gutter_width = 0.0;
        egui::TopBottomPanel::bottom("status_line")
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame {
                fill: self.state.preferences.theme.status_bar_bg.into(),
                inner_margin: egui::Margin::symmetric(8.0, 8.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.memory_mut(|mem| {
                    if let Some(id) = mem.focused() {
                        mem.surrender_focus(id);
                    }
                });
                if self.state.buffer_idx.is_some() {
                    let (buffer, instance) =
                        self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
                    let file_path = buffer.file_path.clone();
                    let modified = buffer.modified;
                    let cursor = instance.cursor;

                    ui.horizontal(|ui| {
                        let mode = &self.state.mode;
                        match mode {
                            Mode::Normal => ui.label(
                                RichText::new("NORMAL")
                                    .color(self.state.preferences.theme.status_bar_normal_mode_fg),
                            ),
                            Mode::Insert => ui.label(
                                RichText::new("INSERT")
                                    .color(self.state.preferences.theme.status_bar_insert_mode_fg),
                            ),
                        };
                        ui.separator();
                        ui.label(file_path.as_ref().unwrap());
                        ui.separator();
                        ui.label(format!("{}:{}", cursor.row + 1, cursor.column + 1));
                        ui.separator();
                        ui.label(if modified { "U" } else { "" });
                        ui.separator();
                        if ui.button("+").clicked() {
                            self.state.preferences.editor_font_size += 1;
                        };
                        ui.label(format!(
                            "Font Size: {}",
                            self.state.preferences.editor_font_size
                        ));
                        if ui.button("-").clicked() {
                            self.state.preferences.editor_font_size -= 1;
                        };
                        ui.separator();
                        if ui
                            .button(format!("Tab Size: {}", self.state.preferences.tab_width))
                            .clicked()
                        {
                            if self.state.preferences.tab_width == 4 {
                                self.state.preferences.tab_width = 2;
                            } else {
                                self.state.preferences.tab_width = 4;
                            }
                        };
                        ui.separator();
                        if ui
                            .button(
                                (if self.state.preferences.line_ending == "\n" {
                                    "lf"
                                } else {
                                    "crlf"
                                })
                                .to_string(),
                            )
                            .clicked()
                        {
                            if self.state.preferences.line_ending == "\n" {
                                self.state.preferences.line_ending = "\r\n".to_string()
                            } else {
                                self.state.preferences.line_ending = "\n".to_string();
                            }
                        };
                        ui.separator();
                    });
                }
            });
        egui::SidePanel::left("gutter")
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame {
                fill: self.state.preferences.theme.gutter_bg.into(),
                inner_margin: egui::Margin::same(self.state.preferences.gutter_padding),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                let rect = ui.max_rect();
                gutter_width = rect.width() + self.state.preferences.gutter_padding * 2.0;
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
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: self.state.preferences.theme.editor_bg.into(),
                inner_margin: egui::Margin::same(self.state.preferences.editor_padding),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let label_response = ui.label(RichText::new("x").font(FontId::monospace(
                    self.state.preferences.editor_font_size as f32,
                )));
                char_width = label_response.rect.width();
                char_height = label_response.rect.height();
            });
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: self.state.preferences.theme.editor_bg.into(),
                outer_margin: egui::Margin::same(self.state.preferences.editor_padding),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                let rect = ui.max_rect();
                let visible_lines = (rect.height() / char_height).floor() as usize;
                let max_characters = (rect.width() / char_width).floor() as usize;

                if let Ok(async_result) = self.state.async_handle.receiver.try_recv() {
                    let (buffer, _instance) =
                        self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
                    let mut lsp_handle = self.lsp_handles.get_mut(&buffer.language);
                    (async_result.callback)(async_result.result, &mut self.state, &mut lsp_handle);
                }

                if self.state.buffer_idx.is_some() {
                    let (buffer, _instance) =
                        self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
                    if let Some(lsp_handle) = self.lsp_handles.get_mut(&buffer.language) {
                        if let Some(message) = lsp_handle.recv_message_sync() {
                            match message {
                                rift_core::lsp::client::IncomingMessage::Response(response) => {
                                    if response.error.is_some() {
                                        tracing::error!(
                                            "---Error: Message Id: {}\n\n{:#?}---\n",
                                            response.id,
                                            response.error.unwrap()
                                        );
                                    } else if lsp_handle.id_method[&response.id]
                                        == "textDocument/hover"
                                        && response.result.is_some()
                                    {
                                        let message = response.result.unwrap()["contents"]["value"]
                                            .as_str()
                                            .unwrap()
                                            .to_string();
                                        self.info_modal.info = message;
                                        self.info_modal.active = true;
                                        self.editor_focused = false;
                                    } else if lsp_handle.id_method[&response.id]
                                        == "textDocument/completion"
                                        && response.result.is_some()
                                    {
                                        let items = response.result.unwrap()["items"]
                                            .as_array()
                                            .unwrap()
                                            .clone();
                                        let mut completion_items = vec![];
                                        for item in items {
                                            completion_items.push(types::CompletionItem {
                                                label: item["label"].as_str().unwrap().to_owned(),
                                                edit: types::TextEdit {
                                                    text: item["textEdit"]["newText"]
                                                        .as_str()
                                                        .unwrap()
                                                        .to_owned(),
                                                    range: Selection {
                                                        cursor: Cursor {
                                                            row: item["textEdit"]["range"]["end"]
                                                                ["line"]
                                                                .as_u64()
                                                                .unwrap()
                                                                as usize,
                                                            column: item["textEdit"]["range"]["end"]
                                                                ["character"]
                                                                .as_u64()
                                                                .unwrap()
                                                                as usize,
                                                        },
                                                        mark: Cursor {
                                                            row: item["textEdit"]["range"]["start"]
                                                                ["line"]
                                                                .as_u64()
                                                                .unwrap()
                                                                as usize,
                                                            column: item["textEdit"]["range"]
                                                                ["start"]["character"]
                                                                .as_u64()
                                                                .unwrap()
                                                                as usize,
                                                        },
                                                    },
                                                },
                                            });
                                        }
                                        self.completion_menu.set_items(completion_items);
                                        self.completion_menu.active = true;
                                        self.editor_focused = false;
                                    } else if lsp_handle.id_method[&response.id]
                                        == "textDocument/formatting"
                                        && response.result.is_some()
                                    {
                                        let edits =
                                            response.result.unwrap().as_array().unwrap().clone();
                                        for edit in edits {
                                            let text_edit = types::TextEdit {
                                                text: edit["newText"].as_str().unwrap().to_owned(),
                                                range: Selection {
                                                    cursor: Cursor {
                                                        row: edit["range"]["end"]["line"]
                                                            .as_u64()
                                                            .unwrap()
                                                            as usize,
                                                        column: edit["range"]["end"]["character"]
                                                            .as_u64()
                                                            .unwrap()
                                                            as usize,
                                                    },
                                                    mark: Cursor {
                                                        row: edit["range"]["start"]["line"]
                                                            .as_u64()
                                                            .unwrap()
                                                            as usize,
                                                        column: edit["range"]["start"]["character"]
                                                            .as_u64()
                                                            .unwrap()
                                                            as usize,
                                                    },
                                                },
                                            };
                                            perform_action(
                                                Action::DeleteText(text_edit.range),
                                                &mut self.state,
                                                &mut Some(lsp_handle),
                                            );
                                            perform_action(
                                                Action::InsertText(
                                                    text_edit.text,
                                                    text_edit.range.mark,
                                                ),
                                                &mut self.state,
                                                &mut Some(lsp_handle),
                                            );
                                        }
                                    } else {
                                        let message = format!(
                                            "---Response to: {}({})\n\n{:#?}---\n",
                                            lsp_handle.id_method[&response.id],
                                            response.id,
                                            response.result
                                        );
                                        tracing::info!("{}", message);
                                    }
                                }
                                rift_core::lsp::client::IncomingMessage::Notification(
                                    notification,
                                ) => {
                                    if notification.method == "textDocument/publishDiagnostics"
                                        && notification.params.is_some()
                                    {
                                        let mut uri = std::path::absolute(
                                            notification.params.as_ref().unwrap()["uri"]
                                                .as_str()
                                                .unwrap()
                                                .strip_prefix("file:")
                                                .unwrap()
                                                .trim_start_matches("\\"),
                                        )
                                        .unwrap()
                                        .to_str()
                                        .unwrap()
                                        .to_string();
                                        #[cfg(target_os = "windows")]
                                        {
                                            uri = uri.to_lowercase();
                                        }

                                        let mut diagnostics = types::PublishDiagnostics {
                                            uri,
                                            version: notification.params.as_ref().unwrap()
                                                ["version"]
                                                .as_u64()
                                                .unwrap_or(0)
                                                as usize,
                                            diagnostics: vec![],
                                        };

                                        for diagnostic in notification.params.as_ref().unwrap()
                                            ["diagnostics"]
                                            .as_array()
                                            .unwrap()
                                        {
                                            diagnostics.diagnostics.push(types::Diagnostic {
                                                range: Selection {
                                                    cursor: Cursor {
                                                        row: diagnostic["range"]["end"]["line"]
                                                            .as_u64()
                                                            .unwrap()
                                                            as usize,
                                                        column: diagnostic["range"]["end"]
                                                            ["character"]
                                                            .as_u64()
                                                            .unwrap()
                                                            as usize,
                                                    },
                                                    mark: Cursor {
                                                        row: diagnostic["range"]["start"]["line"]
                                                            .as_u64()
                                                            .unwrap()
                                                            as usize,
                                                        column: diagnostic["range"]["start"]
                                                            ["character"]
                                                            .as_u64()
                                                            .unwrap()
                                                            as usize,
                                                    },
                                                },
                                                severity: match diagnostic["severity"]
                                                    .as_u64()
                                                    .unwrap_or(1)
                                                {
                                                    1 => types::DiagnosticSeverity::Error,
                                                    2 => types::DiagnosticSeverity::Warning,
                                                    3 => types::DiagnosticSeverity::Information,
                                                    4 => types::DiagnosticSeverity::Hint,
                                                    _ => types::DiagnosticSeverity::Error,
                                                },
                                                code: diagnostic["code"].to_string(),
                                                source: diagnostic["source"].to_string(),
                                                message: diagnostic["message"].to_string(),
                                            });
                                        }
                                        self.state
                                            .diagnostics
                                            .insert(diagnostics.uri.clone(), diagnostics);
                                    } else {
                                        let message = format!(
                                            "---Notification: {}\n\n{:#?}---\n",
                                            notification.method, notification.params
                                        );
                                        tracing::info!("{}", message);
                                        self.diagnostics_overlay.info = message;
                                    }
                                }
                            }
                        }
                    }
                }

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
                        let mut format = egui::TextFormat {
                            font_id: FontId::monospace(
                                self.state.preferences.editor_font_size as f32,
                            ),
                            ..Default::default()
                        };
                        for attribute in &token.1 {
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
                                Attribute::Cursor => {}
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

                if self.editor_focused {
                    self.dispatcher
                        .show(ui, &mut self.state, &mut self.lsp_handles);
                }
            });
        self.editor_focused = self.info_modal.show(ctx);
        if self.state.buffer_idx.is_some() {
            let (buffer, _instance) = self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
            let lsp_handle = &mut self.lsp_handles.get_mut(&buffer.language);
            self.editor_focused =
                self.editor_focused && self.completion_menu.show(ctx, &mut self.state, lsp_handle);
        }
        self.diagnostics_overlay.show(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::TRANSPARENT,
                outer_margin: egui::Margin::same(self.state.preferences.editor_padding),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.put(
                    Rect::from_two_pos(
                        egui::Pos2 {
                            x: (self.state.relative_cursor.column as f32 * char_width)
                                + gutter_width
                                + self.state.preferences.editor_padding,
                            y: (self.state.relative_cursor.row as f32 * char_height)
                                + self.state.preferences.editor_padding,
                        },
                        egui::Pos2 {
                            x: (self.state.relative_cursor.column as f32 * char_width)
                                + gutter_width
                                + char_width
                                + self.state.preferences.editor_padding,
                            y: (self.state.relative_cursor.row as f32 * char_height)
                                + char_height
                                + self.state.preferences.editor_padding,
                        },
                    ),
                    Label::new(
                        RichText::new(" ")
                            .font(FontId::monospace(
                                self.state.preferences.editor_font_size as f32,
                            ))
                            .background_color(if matches!(self.state.mode, Mode::Normal) {
                                self.state.preferences.theme.cursor_normal_mode_bg
                            } else {
                                self.state.preferences.theme.cursor_insert_mode_bg
                            }),
                    ),
                );
            });
        if self.state.modal.open {
            egui::Window::new("modal")
                .movable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .frame(egui::Frame {
                    fill: self.state.preferences.theme.modal_bg.into(),
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.label(&self.state.modal.input);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (idx, entry) in self.state.modal.options.iter().enumerate() {
                            ui.label(
                                RichText::new(&entry.0)
                                    .color(
                                        if self.state.modal.selection.is_some()
                                            && idx == self.state.modal.selection.unwrap()
                                        {
                                            self.state.preferences.theme.modal_active
                                        } else {
                                            self.state.preferences.theme.modal_text
                                        },
                                    )
                                    .size(self.state.preferences.ui_font_size as f32),
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
            let (buffer, _instance) = self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
            let mut extra_segments = vec![];
            let mut path = buffer.file_path.as_ref().unwrap().clone();
            #[cfg(target_os = "windows")]
            {
                path = path.to_lowercase();
            }

            if let Some(diagnostics) = self.state.diagnostics.get(&path) {
                if diagnostics.version != 0 && diagnostics.version == buffer.version {
                    for diagnostic in &diagnostics.diagnostics {
                        extra_segments.push(Range {
                            start: buffer.byte_index_from_cursor(&diagnostic.range.mark, "\n"),
                            end: buffer.byte_index_from_cursor(&diagnostic.range.cursor, "\n"),
                            attributes: HashSet::from([Attribute::DiagnosticSeverity(
                                diagnostic.severity.clone(),
                            )]),
                        });
                    }
                }
            }
            let (buffer, instance) = self
                .state
                .get_buffer_by_id_mut(self.state.buffer_idx.unwrap());
            let (lines, relative_cursor, gutter_info) = buffer.get_visible_lines(
                &mut instance.scroll,
                &instance.cursor,
                &instance.selection,
                visible_lines,
                max_characters,
                "\n".into(),
                extra_segments,
            );
            self.state.highlighted_text = lines;
            self.state.gutter_info = gutter_info;
            return relative_cursor;
        }
        rift_core::buffer::instance::Cursor { row: 0, column: 0 }
    }
}

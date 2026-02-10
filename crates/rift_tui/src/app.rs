use std::{collections::HashSet, time::Duration};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text, widgets,
};

use rift_core::{
    actions::{Action, perform_action},
    buffer::instance::Attribute,
    io::file_io::handle_file_event,
    lsp::handle_lsp_messages,
    rendering::update_visible_lines,
    state::{CompletionMenu, EditorState, Mode},
};

use crate::util::color_from_rgb;

pub(crate) struct App {
    pub state: EditorState,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub(crate) fn new() -> Self {
        let mut state = EditorState::new();
        state.post_initialization();

        Self { state }
    }

    fn perform_action(&mut self, action: Action) -> String {
        perform_action(action, &mut self.state).unwrap_or_default()
    }

    pub(crate) fn run(&mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while !self.state.quit {
            terminal.draw(|frame| {
                let show_gutter = !matches!(self.state.is_active_buffer_special(), Some(true));
                let gutter_width = if show_gutter {
                    if let Some(buffer_idx) = self.state.buffer_idx {
                        let (buffer, _) = self.state.get_buffer_by_id(buffer_idx);
                        let line_count = buffer.get_num_lines().max(1);
                        let digits = line_count.to_string().len();
                        ((digits + 2).max(3)) as u16
                    } else {
                        0
                    }
                } else {
                    0
                };
                let v_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Fill(1), Constraint::Length(1)])
                    .split(frame.area());
                let h_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(if show_gutter {
                        vec![Constraint::Length(gutter_width), Constraint::Fill(1)]
                    } else {
                        vec![Constraint::Length(0), Constraint::Fill(1)]
                    })
                    .split(v_layout[0]);

                let viewport_rows = h_layout[1].height as usize;
                let viewport_columns = h_layout[1].width as usize;

                // Update if resized
                if self
                    .state
                    .set_viewport_size(viewport_rows, viewport_columns)
                {
                    self.state.update_view = true;
                    if self.state.init_rsl_complete {
                        let _ = self.perform_action(Action::RunSource(format!(
                            "onViewportSizeChanged({}, {})",
                            viewport_rows, viewport_columns
                        )));
                    }
                }

                if let Ok(async_result) = self.state.async_handle.receiver.try_recv() {
                    (async_result.callback)(async_result.result, &mut self.state);
                    self.state.update_view = true;
                }

                while let Ok(action_request) = self.state.event_reciever.try_recv() {
                    let result = self.perform_action(action_request.action);
                    action_request.response_tx.send(result).unwrap();
                    self.state.update_view = true;
                    std::thread::sleep(Duration::from_millis(1));
                }

                // Handle file watcher events
                if let Ok(file_event_result) = self.state.file_event_receiver.try_recv() {
                    handle_file_event(file_event_result, &mut self.state);
                    self.state.update_view = true;
                }

                handle_lsp_messages(&mut self.state);

                if self.state.buffer_idx.is_some() {
                    // Compute view if updated
                    if self.state.update_view {
                        self.state.relative_cursor =
                            update_visible_lines(&mut self.state, viewport_rows, viewport_columns);
                        self.state.update_view = false;
                    }

                    // Render text
                    let mut lines = vec![];
                    for line in &self.state.highlighted_text {
                        let mut line_widget = vec![];
                        for token in line {
                            let mut style = Style::new();
                            let mut attributes: Vec<&Attribute> = token.1.iter().collect();
                            attributes.sort();
                            for attribute in attributes {
                                match attribute {
                                    Attribute::None => {}
                                    Attribute::Visible => {}
                                    Attribute::Underline => {}
                                    Attribute::Highlight(highlight_type) => match highlight_type {
                                        rift_core::buffer::instance::HighlightType::None => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_none,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::White => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_white,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::Red => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_red,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::Orange => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_orange,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::Blue => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_blue,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::Green => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_green,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::Purple => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_purple,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::Yellow => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_yellow,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::Gray => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_gray,
                                            ));
                                        }
                                        rift_core::buffer::instance::HighlightType::Turquoise => {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.highlight_turquoise,
                                            ));
                                        }
                                    },
                                    Attribute::Select => {
                                        style = style.bg(color_from_rgb(
                                            self.state.preferences.theme.selection_bg,
                                        ));
                                    }
                                    Attribute::Cursor => {
                                        if matches!(self.state.mode, Mode::Normal) {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.cursor_normal_mode_fg,
                                            ));
                                            style = style.bg(color_from_rgb(
                                                self.state.preferences.theme.cursor_normal_mode_bg,
                                            ));
                                        } else {
                                            style = style.fg(color_from_rgb(
                                                self.state.preferences.theme.cursor_insert_mode_fg,
                                            ));
                                            style = style.bg(color_from_rgb(
                                                self.state.preferences.theme.cursor_insert_mode_bg,
                                            ));
                                        }
                                    }
                                    Attribute::DiagnosticSeverity(_severity) => {
                                        style = style.add_modifier(Modifier::UNDERLINED);
                                        // .underline_color(color_from_rgb(match severity {
                                        //     types::DiagnosticSeverity::Error => {
                                        //         self.preferences.theme.error
                                        //     }
                                        //     types::DiagnosticSeverity::Warning => {
                                        //         self.preferences.theme.warning
                                        //     }
                                        //     types::DiagnosticSeverity::Information => {
                                        //         self.preferences.theme.information
                                        //     }
                                        //     types::DiagnosticSeverity::Hint => {
                                        //         self.preferences.theme.hint
                                        //     }
                                        // }));
                                    }
                                }
                            }
                            line_widget.push(text::Span::styled(&token.0, style));
                        }
                        lines.push(text::Line::from(line_widget));
                    }

                    let editor_style =
                        Style::new().bg(color_from_rgb(self.state.preferences.theme.editor_bg));
                    let editor_text =
                        widgets::Paragraph::new(text::Text::from(lines)).style(editor_style);
                    frame.render_widget(editor_text, h_layout[1]);

                    if show_gutter {
                        // Render gutter
                        let mut gutter_lines = vec![];
                        for (idx, gutter_line) in self.state.gutter_info.iter().enumerate() {
                            let gutter_value = if gutter_line.wrapped {
                                ".  ".to_string()
                            } else {
                                format!("{}  ", gutter_line.start.row + 1)
                            };
                            if idx == self.state.relative_cursor.row {
                                gutter_lines.push(
                                    text::Line::styled(
                                        gutter_value,
                                        Style::new()
                                            .fg(color_from_rgb(
                                                self.state
                                                    .preferences
                                                    .theme
                                                    .gutter_text_current_line,
                                            ))
                                            .bg(color_from_rgb(
                                                self.state.preferences.theme.gutter_current_line_bg,
                                            )),
                                    )
                                    .alignment(ratatui::layout::Alignment::Right),
                                );
                            } else {
                                gutter_lines.push(
                                    text::Line::styled(
                                        gutter_value,
                                        Style::new().fg(color_from_rgb(
                                            self.state.preferences.theme.gutter_text,
                                        )),
                                    )
                                    .alignment(ratatui::layout::Alignment::Right),
                                );
                            }
                        }
                        let gutter_style = Style::new()
                            .bg(color_from_rgb(self.state.preferences.theme.gutter_bg))
                            .fg(color_from_rgb(self.state.preferences.theme.gutter_text));
                        let gutter_text = widgets::Paragraph::new(text::Text::from(gutter_lines))
                            .style(gutter_style)
                            .alignment(ratatui::layout::Alignment::Right);
                        frame.render_widget(gutter_text, h_layout[0]);
                    }

                    // Render status line
                    let status_mode_style = Style::default()
                        .fg(color_from_rgb(self.state.preferences.theme.status_bar_bg))
                        .bg(color_from_rgb(if matches!(self.state.mode, Mode::Normal) {
                            self.state.preferences.theme.status_bar_normal_mode_fg
                        } else {
                            self.state.preferences.theme.status_bar_insert_mode_fg
                        }));
                    let status_bar_style = Style::new()
                        .bg(color_from_rgb(self.state.preferences.theme.status_bar_bg))
                        .fg(color_from_rgb(self.state.preferences.theme.ui_text));
                    let (buffer, instance) =
                        self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
                    let file_label = buffer
                        .display_name
                        .clone()
                        .unwrap_or(self.state.buffer_idx.unwrap().to_string());
                    let mut left_spans = vec![text::Span::styled(
                        format!(" {:#?} ", self.state.mode),
                        status_mode_style,
                    )];
                    if self.state.audio_recording {
                        left_spans.push(text::Span::styled(
                            " ⏺ REC ".to_string(),
                            Style::default().fg(color_from_rgb(self.state.preferences.theme.error)),
                        ));
                    }
                    let left_file = format!(" {} ", file_label);
                    left_spans.push(left_file.into());

                    let cursor_label = format!(
                        " {}:{} ",
                        instance.cursor.row + 1,
                        instance.cursor.column + 1,
                    );
                    let modified_label = format!(" {} ", if buffer.modified { "U" } else { "" });
                    let keybind_label =
                        format!(" {} ", self.state.keybind_handler.running_sequence);
                    let log_label = format!(
                        " {} ",
                        self.state.log_messages.last().unwrap_or(&String::new())
                    );
                    let right_len = cursor_label.chars().count()
                        + modified_label.chars().count()
                        + keybind_label.chars().count()
                        + log_label.chars().count();
                    let status_area_width = v_layout[1].width as usize;
                    let max_right = status_area_width.saturating_sub(10).max(1);
                    let right_width = right_len.min(max_right).min(status_area_width) as u16;
                    let status_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(1), Constraint::Length(right_width)])
                        .split(v_layout[1]);
                    let left_status = widgets::Paragraph::new(text::Line::from(left_spans))
                        .style(status_bar_style);
                    let right_status = widgets::Paragraph::new(text::Line::from(vec![
                        cursor_label.into(),
                        modified_label.into(),
                        keybind_label.into(),
                        log_label.into(),
                    ]))
                    .style(status_bar_style)
                    .alignment(ratatui::layout::Alignment::Right);
                    frame.render_widget(left_status, status_layout[0]);
                    frame.render_widget(right_status, status_layout[1]);
                }

                // Render diagnostics overlay
                if self.state.diagnostics_overlay.should_render() {
                    let area = Rect {
                        x: frame.area().width * 7 / 8 - 4,
                        y: 2,
                        width: frame.area().width / 8,
                        height: frame.area().height - 4,
                    };
                    let diagnostics_info =
                        widgets::Paragraph::new(self.state.diagnostics_overlay.content.clone())
                            .wrap(widgets::Wrap { trim: false });
                    frame.render_widget(diagnostics_info, area);
                }

                // Render signature information
                if self.state.signature_information.should_render()
                    && self.state.relative_cursor.row > 1
                    && self.state.relative_cursor.row < self.state.viewport_rows().saturating_sub(1)
                {
                    let popup_area = Rect {
                        x: self.state.relative_cursor.column as u16 + h_layout[1].x + 1,
                        y: self.state.relative_cursor.row as u16 + h_layout[1].y - 1,
                        width: (self.state.signature_information.content.len() as u16).min(
                            frame.area().width
                                - self.state.relative_cursor.column as u16
                                - h_layout[1].x
                                - 3,
                        ),
                        height: 1,
                    };
                    let signature_block = widgets::Block::default().style(
                        Style::new()
                            .bg(color_from_rgb(self.state.preferences.theme.modal_bg))
                            .fg(color_from_rgb(self.state.preferences.theme.modal_text)),
                    );

                    let signature_information =
                        widgets::Paragraph::new(self.state.signature_information.content.clone())
                            .block(signature_block);
                    frame.render_widget(widgets::Clear, popup_area);
                    frame.render_widget(signature_information, popup_area);
                }

                // Render Completion Items
                if self.state.completion_menu.active {
                    let offset_y = if viewport_rows - self.state.completion_menu.max_items - 1
                        < self.state.relative_cursor.row
                    {
                        self.state.completion_menu.max_items as u16
                    } else {
                        0
                    };
                    let popup_area = Rect {
                        x: (self.state.relative_cursor.column as u16 + h_layout[1].x + 1)
                            .min(frame.area().width - 35),
                        y: self.state.relative_cursor.row as u16 + h_layout[1].y + 1 - offset_y,
                        width: 30,
                        height: self.state.completion_menu.max_items as u16,
                    };
                    let completion_list_block = widgets::Block::default().style(
                        Style::new()
                            .bg(color_from_rgb(self.state.preferences.theme.modal_bg))
                            .fg(color_from_rgb(self.state.preferences.theme.modal_text)),
                    );

                    let completion_list = self
                        .state
                        .completion_menu
                        .items
                        .iter()
                        .map(|item| item.label.clone())
                        .collect::<widgets::List>()
                        .highlight_style(
                            Style::new()
                                .fg(color_from_rgb(self.state.preferences.theme.modal_primary)),
                        )
                        .block(completion_list_block);
                    frame.render_widget(widgets::Clear, popup_area);
                    let mut list_state = widgets::ListState::default();
                    list_state.select(self.state.completion_menu.selection);
                    frame.render_stateful_widget(completion_list, popup_area, &mut list_state);
                }
            })?;

            // Handle keyboard events
            if event::poll(Duration::from_millis(5))?
                && let event::Event::Key(key) = event::read()?
            {
                self.state.update_view = true;
                if key.kind == KeyEventKind::Press {
                    if !(self.state.completion_menu.active
                        && (key.code == KeyCode::Tab || key.code == KeyCode::Enter))
                    {
                        let keybind = match key.code {
                            KeyCode::Backspace => "Backspace",
                            KeyCode::Enter => "Enter",
                            KeyCode::Left => "Left",
                            KeyCode::Right => "Right",
                            KeyCode::Up => "Up",
                            KeyCode::Down => "Down",
                            KeyCode::Home => "Home",
                            KeyCode::End => "End",
                            KeyCode::PageUp => "PageUp",
                            KeyCode::PageDown => "PageDown",
                            KeyCode::Tab => "Tab",
                            KeyCode::Delete => "Delete",
                            KeyCode::Insert => "Insert",
                            KeyCode::F(n) => match n {
                                1 => "F1",
                                2 => "F2",
                                3 => "F3",
                                4 => "F4",
                                5 => "F5",
                                6 => "F6",
                                7 => "F7",
                                8 => "F8",
                                9 => "F9",
                                10 => "F10",
                                11 => "F11",
                                12 => "F12",
                                _ => "",
                            },
                            KeyCode::Char(c) => {
                                if c == ' ' {
                                    "Space"
                                } else if c.is_ascii() {
                                    &c.to_string()
                                } else {
                                    ""
                                }
                            }
                            KeyCode::Esc => "Escape",
                            _ => "",
                        };
                        let mut modifiers_set = HashSet::new();
                        if key.modifiers.contains(KeyModifiers::ALT) {
                            modifiers_set.insert("m".to_string());
                        } else if key.modifiers.contains(KeyModifiers::CONTROL) {
                            modifiers_set.insert("c".to_string());
                        } else if key.modifiers.contains(KeyModifiers::SHIFT) {
                            modifiers_set.insert("s".to_string());
                        }

                        if let Some(action) = self.state.keybind_handler.handle_input(
                            self.state.buffer_idx,
                            self.state.is_active_buffer_special(),
                            self.state.mode.clone(),
                            keybind.to_string(),
                            modifiers_set,
                        ) {
                            perform_action(action, &mut self.state);
                        }
                    }

                    if self.state.completion_menu.active {
                        if key.code == KeyCode::Esc {
                            self.state.completion_menu.close();
                            self.state.signature_information.content = String::new();
                        } else if key.code == KeyCode::Tab {
                            self.state.completion_menu.select_next();
                        } else if key.code == KeyCode::Enter {
                            let completion_item = self.state.completion_menu.select();
                            CompletionMenu::on_select(completion_item, &mut self.state);
                            self.state.signature_information.content = String::new();
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

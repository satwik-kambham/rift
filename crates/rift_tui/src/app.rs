use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text,
    widgets::{self},
    DefaultTerminal,
};
use rift_core::{
    actions::{perform_action, Action},
    buffer::instance::{Attribute, Language},
    cli::{process_cli_args, CLIArgs},
    lsp::{client::LSPClientHandle, handle_lsp_messages},
    preferences::Color,
    rendering::update_visible_lines,
    state::{CompletionMenu, EditorState, Mode},
};

pub fn color_from_rgb(c: Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(c.r, c.g, c.b)
}

pub struct App {
    pub state: EditorState,
    pub lsp_handles: HashMap<Language, LSPClientHandle>,
    pub modal_list_state: widgets::ListState,
    pub info_modal_scroll: u16,
}

impl App {
    pub fn new(rt: tokio::runtime::Runtime, cli_args: CLIArgs) -> Self {
        let mut state = EditorState::new(rt);
        let mut lsp_handles = HashMap::new();

        process_cli_args(cli_args, &mut state, &mut lsp_handles);

        Self {
            state,
            lsp_handles,
            modal_list_state: widgets::ListState::default(),
            info_modal_scroll: 0,
        }
    }

    pub fn perform_action(&mut self, action: Action) {
        perform_action(action, &mut self.state, &mut self.lsp_handles);
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while !self.state.quit {
            terminal.draw(|frame| {
                // Layout
                let v_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Fill(1), Constraint::Length(1)])
                    .split(frame.area());
                let h_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(7), Constraint::Fill(1)])
                    .split(v_layout[0]);

                let visible_lines = h_layout[1].height as usize;
                let max_characters = h_layout[1].width as usize;

                // Update if resized
                if visible_lines != self.state.visible_lines
                    || max_characters != self.state.max_characters
                {
                    self.state.visible_lines = visible_lines;
                    self.state.max_characters = max_characters;
                    self.state.update_view = true;
                }

                if let Ok(async_result) = self.state.async_handle.receiver.try_recv() {
                    (async_result.callback)(
                        async_result.result,
                        &mut self.state,
                        &mut self.lsp_handles,
                    );
                }

                handle_lsp_messages(&mut self.state, &mut self.lsp_handles);

                if self.state.buffer_idx.is_some() {
                    // Compute view if updated
                    if self.state.update_view {
                        self.state.relative_cursor =
                            update_visible_lines(&mut self.state, visible_lines, max_characters);
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

                    frame.render_widget(text::Text::from(lines), h_layout[1]);

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
                                    Style::new().fg(color_from_rgb(
                                        self.state.preferences.theme.gutter_text_current_line,
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
                    frame.render_widget(text::Text::from(gutter_lines), h_layout[0]);

                    // Render status line
                    let status_mode_style = Style::default()
                        .fg(color_from_rgb(self.state.preferences.theme.status_bar_bg))
                        .bg(color_from_rgb(if matches!(self.state.mode, Mode::Normal) {
                            self.state.preferences.theme.status_bar_normal_mode_fg
                        } else {
                            self.state.preferences.theme.status_bar_insert_mode_fg
                        }));
                    let status = text::Line::from(vec![
                        text::Span::styled(format!(" {:#?} ", self.state.mode), status_mode_style),
                        format!(
                            " {} ",
                            self.state
                                .get_buffer_by_id(self.state.buffer_idx.unwrap())
                                .0
                                .file_path
                                .as_ref()
                                .unwrap(),
                        )
                        .into(),
                        format!(
                            " {}:{} ",
                            self.state
                                .get_buffer_by_id(self.state.buffer_idx.unwrap())
                                .1
                                .cursor
                                .row
                                + 1,
                            self.state
                                .get_buffer_by_id(self.state.buffer_idx.unwrap())
                                .1
                                .cursor
                                .column
                                + 1,
                        )
                        .into(),
                        format!(
                            " {} ",
                            if self
                                .state
                                .get_buffer_by_id(self.state.buffer_idx.unwrap())
                                .0
                                .modified
                            {
                                "U"
                            } else {
                                ""
                            },
                        )
                        .into(),
                        format!(" {} ", self.state.keybind_handler.running_sequence).into(),
                    ]);
                    frame.render_widget(status, v_layout[1]);
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
                    && self.state.relative_cursor.row < self.state.visible_lines - 1
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

                // Render Modal
                if self.state.modal.open {
                    let popup_area = Rect {
                        x: 4,
                        y: 2,
                        width: frame.area().width - 8,
                        height: frame.area().height - 4,
                    };
                    let modal_block = widgets::Block::default()
                        .borders(widgets::Borders::ALL)
                        .border_style(
                            Style::new()
                                .fg(color_from_rgb(self.state.preferences.theme.ui_bg_stroke)),
                        )
                        .style(
                            Style::new()
                                .bg(color_from_rgb(self.state.preferences.theme.modal_bg))
                                .fg(color_from_rgb(self.state.preferences.theme.modal_text)),
                        );

                    let modal_layout = Layout::vertical([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(1),
                    ])
                    .split(modal_block.inner(popup_area));
                    let modal_list = self
                        .state
                        .modal
                        .options
                        .iter()
                        .map(|option| option.0.clone())
                        .collect::<widgets::List>()
                        .highlight_spacing(widgets::HighlightSpacing::Always)
                        .highlight_symbol(" > ")
                        .highlight_style(
                            Style::new()
                                .fg(color_from_rgb(self.state.preferences.theme.modal_primary)),
                        );
                    frame.render_widget(widgets::Clear, popup_area);
                    frame.render_widget(modal_block, popup_area);
                    frame.render_widget(&self.state.modal.input, modal_layout[0]);
                    frame.render_stateful_widget(
                        modal_list,
                        modal_layout[2],
                        &mut self.modal_list_state,
                    );
                }

                // Render Completion Items
                if self.state.completion_menu.active {
                    let offset_y = if visible_lines - self.state.completion_menu.max_items - 1
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

                // Render Info Modal
                if self.state.info_modal.active {
                    let popup_area = Rect {
                        x: 4,
                        y: 2,
                        width: frame.area().width - 8,
                        height: frame.area().height - 4,
                    };
                    let info_modal_block = widgets::Block::default()
                        .borders(widgets::Borders::ALL)
                        .border_style(
                            Style::new()
                                .fg(color_from_rgb(self.state.preferences.theme.ui_bg_stroke)),
                        )
                        .style(
                            Style::new()
                                .bg(color_from_rgb(self.state.preferences.theme.modal_bg))
                                .fg(color_from_rgb(self.state.preferences.theme.modal_text)),
                        );
                    let content = widgets::Paragraph::new(&*self.state.info_modal.content)
                        .block(info_modal_block)
                        .wrap(widgets::Wrap { trim: false })
                        .scroll((self.info_modal_scroll, 0));
                    frame.render_widget(widgets::Clear, popup_area);
                    frame.render_widget(content, popup_area);
                }
            })?;

            // Handle keyboard events
            if event::poll(Duration::from_millis(5))? {
                if let event::Event::Key(key) = event::read()? {
                    self.state.update_view = true;
                    if key.kind == KeyEventKind::Press {
                        if self.state.info_modal.active {
                            if key.code == KeyCode::Esc {
                                self.state.info_modal.close();
                                self.info_modal_scroll = 0;
                            } else if key.code == KeyCode::Up {
                                self.info_modal_scroll = self.info_modal_scroll.saturating_sub(1);
                            } else if key.code == KeyCode::Down {
                                self.info_modal_scroll = self.info_modal_scroll.saturating_add(1);
                            }
                        } else if self.state.modal.open {
                            if let KeyCode::Char(char) = key.code {
                                let mut input = self.state.modal.input.clone();
                                input.push(char);
                                self.state.modal.set_input(input.clone());
                                if let Some(on_input) = self.state.modal.on_input {
                                    on_input(&input, &mut self.state, &mut self.lsp_handles);
                                }
                                self.modal_list_state.select(None);
                            } else if key.code == KeyCode::Tab {
                                self.state.modal.select_next();
                                self.modal_list_state.select(self.state.modal.selection);
                            } else if key.code == KeyCode::Backspace {
                                let mut input = self.state.modal.input.clone();
                                input.pop();
                                self.state.modal.set_input(input.clone());
                                if let Some(on_input) = self.state.modal.on_input {
                                    on_input(&input, &mut self.state, &mut self.lsp_handles);
                                }
                                self.modal_list_state.select(None);
                            } else if key.code == KeyCode::Enter {
                                if let Some(on_select) = self.state.modal.on_select {
                                    if let Some(selection) = self.state.modal.selection {
                                        let alt = key.modifiers.contains(KeyModifiers::ALT);
                                        let options = self
                                            .state
                                            .modal
                                            .options
                                            .get(selection)
                                            .unwrap()
                                            .clone();
                                        on_select(
                                            self.state.modal.input.clone(),
                                            &options,
                                            alt,
                                            &mut self.state,
                                            &mut self.lsp_handles,
                                        );
                                    }
                                }
                                self.modal_list_state.select(None);
                            } else if key.code == KeyCode::Esc {
                                self.state.modal.close();
                                self.modal_list_state.select(None);
                            }
                        } else {
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
                                    self.state.mode.clone(),
                                    keybind.to_string(),
                                    modifiers_set,
                                ) {
                                    perform_action(action, &mut self.state, &mut self.lsp_handles);
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
                                    CompletionMenu::on_select(
                                        completion_item,
                                        &mut self.state,
                                        &mut self.lsp_handles,
                                    );
                                    self.state.signature_information.content = String::new();
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

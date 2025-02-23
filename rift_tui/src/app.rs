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
    buffer::instance::{Attribute, Language, Range},
    cli::{process_cli_args, CLIArgs},
    lsp::{client::LSPClientHandle, handle_lsp_messages},
    preferences::Color,
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
    pub completion_menu_state: widgets::ListState,
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
            completion_menu_state: widgets::ListState::default(),
        }
    }

    pub fn perform_action(&mut self, action: Action) {
        perform_action(action, &mut self.state, &mut self.lsp_handles);
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        loop {
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
                            self.update_visible_lines(visible_lines, max_characters);
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
                                    Attribute::DiagnosticSeverity(severity) => {
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

                    // Render cursor
                    // let buf = frame.buffer_mut();
                    // if let Some(cell) = buf.cell_mut((
                    //     self.state.relative_cursor.column as u16 + h_layout[1].x,
                    //     self.state.relative_cursor.row as u16 + h_layout[1].y,
                    // )) {
                    //     if matches!(self.state.mode, Mode::Normal) {
                    //         cell.set_fg(color_from_rgb(
                    //             self.state.preferences.theme.cursor_normal_mode_fg,
                    //         ));
                    //         cell.set_bg(color_from_rgb(
                    //             self.state.preferences.theme.cursor_normal_mode_bg,
                    //         ));
                    //     } else {
                    //         cell.set_fg(color_from_rgb(
                    //             self.state.preferences.theme.cursor_insert_mode_fg,
                    //         ));
                    //         cell.set_bg(color_from_rgb(
                    //             self.state.preferences.theme.cursor_insert_mode_bg,
                    //         ));
                    //     }
                    // }

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
                            " {}({:?}) ",
                            self.state
                                .get_buffer_by_id(self.state.buffer_idx.unwrap())
                                .0
                                .file_path
                                .as_ref()
                                .unwrap(),
                            self.state.buffer_idx
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
                {
                    let popup_area = Rect {
                        x: self.state.relative_cursor.column as u16 + h_layout[1].x + 1,
                        y: self.state.relative_cursor.row as u16 + h_layout[1].y - 1,
                        width: self
                            .state
                            .signature_information
                            .content
                            .len()
                            .min(max_characters.min(self.state.relative_cursor.column) - 1)
                            as u16,
                        height: 1,
                    };
                    let signature_information =
                        widgets::Paragraph::new(self.state.signature_information.content.clone());
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
                    let modal_block = widgets::Block::default().borders(widgets::Borders::ALL);
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
                        .highlight_symbol(">>");
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
                    let offset = if visible_lines - self.state.completion_menu.max_items - 1
                        < self.state.relative_cursor.row
                    {
                        self.state.completion_menu.max_items as u16
                    } else {
                        0
                    };
                    let popup_area = Rect {
                        x: self.state.relative_cursor.column as u16 + h_layout[1].x + 1,
                        y: self.state.relative_cursor.row as u16 + h_layout[1].y + 1 - offset,
                        width: 20,
                        height: self.state.completion_menu.max_items as u16,
                    };
                    let completion_list = self
                        .state
                        .completion_menu
                        .items
                        .iter()
                        .map(|item| item.label.clone())
                        .collect::<widgets::List>()
                        .highlight_symbol(">>");
                    frame.render_widget(widgets::Clear, popup_area);
                    frame.render_stateful_widget(
                        completion_list,
                        popup_area,
                        &mut self.completion_menu_state,
                    );
                }

                // Render Info Modal
                if self.state.info_modal.active {
                    let popup_area = Rect {
                        x: 4,
                        y: 2,
                        width: frame.area().width - 8,
                        height: frame.area().height - 4,
                    };
                    let info_modal_block = widgets::Block::default().borders(widgets::Borders::ALL);
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
                            } else if key.code == KeyCode::Up {
                                self.info_modal_scroll = self.info_modal_scroll.saturating_sub(1);
                            } else if key.code == KeyCode::Down {
                                self.info_modal_scroll = self.info_modal_scroll.saturating_add(1);
                            }
                        } else if self.state.completion_menu.active {
                            if key.code == KeyCode::Esc {
                                self.state.completion_menu.close();
                                self.completion_menu_state.select(None);
                            } else if key.code == KeyCode::Tab {
                                self.state.completion_menu.select_next();
                                self.completion_menu_state
                                    .select(self.state.completion_menu.selection);
                            } else if key.code == KeyCode::Enter {
                                let completion_item = self.state.completion_menu.select();
                                CompletionMenu::on_select(
                                    completion_item,
                                    &mut self.state,
                                    &mut self.lsp_handles,
                                );
                                self.completion_menu_state.select(None);
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
                        } else if matches!(self.state.mode, Mode::Normal) {
                            if key.code == KeyCode::Char('q') {
                                return Ok(());
                            } else if key.code == KeyCode::Char('i') {
                                self.perform_action(Action::EnterInsertMode);
                            } else if key.code == KeyCode::Char('f') {
                                self.perform_action(Action::OpenFile);
                            } else if key.code == KeyCode::Char('F') {
                                // rift_core::ai::ollama_fim(&mut self.state);
                                self.perform_action(Action::FuzzyFindFile(true));
                            } else if key.code == KeyCode::Char('j') {
                                self.perform_action(Action::MoveCursorDown);
                            } else if key.code == KeyCode::Char('J') {
                                self.perform_action(Action::ExtendCursorDown);
                            } else if key.code == KeyCode::Char('k') {
                                self.perform_action(Action::MoveCursorUp);
                            } else if key.code == KeyCode::Char('K') {
                                self.perform_action(Action::ExtendCursorUp);
                            } else if key.code == KeyCode::Char('h') {
                                self.perform_action(Action::MoveCursorLeft);
                            } else if key.code == KeyCode::Char('H') {
                                self.perform_action(Action::ExtendCursorLeft);
                            } else if key.code == KeyCode::Char('l') {
                                self.perform_action(Action::MoveCursorRight);
                            } else if key.code == KeyCode::Char('L') {
                                self.perform_action(Action::ExtendCursorRight);
                            } else if key.code == KeyCode::Down {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorDown);
                                } else {
                                    self.perform_action(Action::MoveCursorDown);
                                }
                            } else if key.code == KeyCode::Up {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorUp);
                                } else {
                                    self.perform_action(Action::MoveCursorUp);
                                }
                            } else if key.code == KeyCode::Left {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorLeft);
                                } else {
                                    self.perform_action(Action::MoveCursorLeft);
                                }
                            } else if key.code == KeyCode::Right {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorRight);
                                } else {
                                    self.perform_action(Action::MoveCursorRight);
                                }
                            } else if key.code == KeyCode::Home {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorLineStart);
                                } else {
                                    self.perform_action(Action::MoveCursorLineStart);
                                }
                            } else if key.code == KeyCode::End {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorLineEnd);
                                } else {
                                    self.perform_action(Action::MoveCursorLineEnd);
                                }
                            } else if key.code == KeyCode::Char('o') {
                                self.perform_action(Action::AddNewLineBelowAndEnterInsertMode);
                            } else if key.code == KeyCode::Char('d') {
                                self.perform_action(Action::DeleteSelection);
                            } else if key.code == KeyCode::Char('D') {
                                self.perform_action(Action::WorkspaceDiagnostics);
                            } else if key.code == KeyCode::Char('x') {
                                self.perform_action(Action::SelectCurrentLine);
                            } else if key.code == KeyCode::Char('X') {
                                self.perform_action(Action::SelectAndExtentCurrentLine);
                            } else if key.code == KeyCode::Char('w') {
                                self.perform_action(Action::SelectTillEndOfWord);
                            } else if key.code == KeyCode::Char('W') {
                                self.perform_action(Action::ExtendSelectTillEndOfWord);
                            } else if key.code == KeyCode::Char('b') {
                                self.perform_action(Action::SwitchBuffer);
                                // self.perform_action(Action::SelectTillStartOfWord);
                            } else if key.code == KeyCode::Char('B') {
                                self.perform_action(Action::ExtendSelectTillStartOfWord);
                            } else if key.code == KeyCode::Char('a') {
                                self.perform_action(Action::InsertAfterSelection);
                            } else if key.code == KeyCode::Backspace {
                                self.perform_action(Action::DeletePreviousCharacter);
                            } else if key.code == KeyCode::Delete {
                                self.perform_action(Action::DeleteNextCharacter);
                            } else if key.code == KeyCode::Char('g') {
                                self.perform_action(Action::GoToBufferStart);
                            } else if key.code == KeyCode::Char('G') {
                                self.perform_action(Action::GoToBufferEnd);
                            } else if key.code == KeyCode::Char('s') {
                                self.perform_action(Action::FormatCurrentBuffer);
                            } else if key.code == KeyCode::Char('S') {
                                self.perform_action(Action::SaveCurrentBuffer);
                            } else if key.code == KeyCode::Char('u') {
                                self.perform_action(Action::Undo);
                            } else if key.code == KeyCode::Char('U') {
                                self.perform_action(Action::Redo);
                            } else if key.code == KeyCode::Char('>') {
                                self.perform_action(Action::AddIndent);
                            } else if key.code == KeyCode::Char('<') {
                                self.perform_action(Action::RemoveIndent);
                            } else if key.code == KeyCode::Char(',') {
                                self.perform_action(Action::CyclePreviousBuffer);
                            } else if key.code == KeyCode::Char('.') {
                                self.perform_action(Action::CycleNextBuffer);
                            } else if key.code == KeyCode::Char('/') {
                                self.perform_action(Action::SearchWorkspace);
                            } else if key.code == KeyCode::Char(';') {
                                self.perform_action(Action::Unselect);
                            } else if key.code == KeyCode::Char(':') {
                                self.perform_action(Action::OpenCommandDispatcher);
                            } else if key.code == KeyCode::Char('z') {
                                self.perform_action(Action::LSPHover);
                            } else if key.code == KeyCode::Char('Z') {
                                self.perform_action(Action::LSPSignatureHelp);
                                self.perform_action(Action::LSPCompletion);
                            } else if key.code == KeyCode::Char('y') {
                                self.perform_action(Action::CopyToRegister);
                            } else if key.code == KeyCode::Char('Y') {
                                self.perform_action(Action::CopyToClipboard);
                            } else if key.code == KeyCode::Char('p') {
                                self.perform_action(Action::PasteFromRegister);
                            } else if key.code == KeyCode::Char('P') {
                                self.perform_action(Action::PasteFromClipboard);
                            }
                        } else if matches!(self.state.mode, Mode::Insert) {
                            if key.code == KeyCode::Esc {
                                self.perform_action(Action::QuitInsertMode);
                                self.state.signature_information.content = String::new();
                            } else if let KeyCode::Char(c) = key.code {
                                self.perform_action(Action::InsertTextAtCursor(c.into()));
                            } else if key.code == KeyCode::Enter {
                                self.perform_action(Action::InsertNewLineAtCursor);
                            } else if key.code == KeyCode::Down {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorDown);
                                } else {
                                    self.perform_action(Action::MoveCursorDown);
                                }
                            } else if key.code == KeyCode::Up {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorUp);
                                } else {
                                    self.perform_action(Action::MoveCursorUp);
                                }
                            } else if key.code == KeyCode::Left {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorLeft);
                                } else {
                                    self.perform_action(Action::MoveCursorLeft);
                                }
                            } else if key.code == KeyCode::Right {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorRight);
                                } else {
                                    self.perform_action(Action::MoveCursorRight);
                                }
                            } else if key.code == KeyCode::Home {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorLineStart);
                                } else {
                                    self.perform_action(Action::MoveCursorLineStart);
                                }
                            } else if key.code == KeyCode::End {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    self.perform_action(Action::ExtendCursorLineEnd);
                                } else {
                                    self.perform_action(Action::MoveCursorLineEnd);
                                }
                            } else if key.code == KeyCode::Backspace {
                                self.perform_action(Action::DeletePreviousCharacter);
                            } else if key.code == KeyCode::Delete {
                                self.perform_action(Action::DeleteNextCharacter);
                            } else if key.code == KeyCode::Tab {
                                self.perform_action(Action::AddTab);
                            } else {
                                // println!("{:#?}", key.code);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn update_visible_lines(
        &mut self,
        visible_lines: usize,
        max_characters: usize,
    ) -> rift_core::buffer::instance::Cursor {
        if self.state.buffer_idx.is_some() {
            let (buffer, instance) = self.state.get_buffer_by_id(self.state.buffer_idx.unwrap());
            let mut extra_segments = vec![];
            let mut path = buffer.file_path.as_ref().unwrap().clone();
            #[cfg(target_os = "windows")]
            {
                path = path.to_lowercase();
            }

            if let Some(diagnostics) = self.state.diagnostics.get(&path) {
                let mut diagnostic_info = String::new();
                if diagnostics.version != 0 && diagnostics.version == buffer.version {
                    for diagnostic in &diagnostics.diagnostics {
                        if instance.cursor >= diagnostic.range.mark
                            && instance.cursor <= diagnostic.range.cursor
                        {
                            diagnostic_info.push_str(&format!(
                                "{} {} {}\n",
                                diagnostic.source, diagnostic.code, diagnostic.message
                            ));
                        }

                        extra_segments.push(Range {
                            start: buffer.byte_index_from_cursor(&diagnostic.range.mark, "\n"),
                            end: buffer.byte_index_from_cursor(&diagnostic.range.cursor, "\n"),
                            attributes: HashSet::from([Attribute::DiagnosticSeverity(
                                diagnostic.severity.clone(),
                            )]),
                        });
                    }
                }
                self.state.diagnostics_overlay.content = diagnostic_info;
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

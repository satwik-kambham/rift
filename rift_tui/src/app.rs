use std::time::Duration;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text,
    widgets::{self},
    DefaultTerminal,
};
use rift_core::{
    actions::{perform_action, Action},
    buffer::{
        instance::{Cursor, Selection},
        line_buffer::LineBuffer,
    },
    io::file_io,
    lsp::{
        client::{start_lsp, LSPClientHandle},
        types,
    },
    preferences::{Color, Preferences},
    state::{EditorState, Mode},
};

use crate::cli;

pub fn color_from_rgb(c: Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(c.r, c.g, c.b)
}

pub struct App {
    pub state: EditorState,
    pub preferences: Preferences,
    pub lsp_handle: LSPClientHandle,
    pub modal_list_state: widgets::ListState,
    pub rt: tokio::runtime::Runtime,
}

impl App {
    pub fn new(rt: tokio::runtime::Runtime, cli_args: cli::CLIArgs) -> Self {
        let mut lsp_handle = rt.block_on(async { start_lsp().await.unwrap() });
        let mut state = EditorState::default();

        if let Some(path) = cli_args.path {
            let mut path = path;
            if path.is_relative() {
                path = std::path::absolute(path).unwrap();
            }
            if path.is_dir() {
                state.workspace_folder = path.into_os_string().into_string().unwrap();
                lsp_handle.init_lsp_sync(state.workspace_folder.clone());
            } else {
                state.workspace_folder = path.parent().unwrap().to_str().unwrap().to_string();
                lsp_handle.init_lsp_sync(state.workspace_folder.clone());
                let initial_text = file_io::read_file_content(path.to_str().unwrap()).unwrap();
                let buffer = LineBuffer::new(
                    initial_text.clone(),
                    Some(path.to_str().unwrap().to_string()),
                );
                state.buffer_idx = Some(state.add_buffer(buffer));

                lsp_handle
                    .send_notification_sync(
                        "textDocument/didOpen".to_string(),
                        Some(LSPClientHandle::did_open_text_document(
                            path.to_str().unwrap().to_string(),
                            initial_text,
                        )),
                    )
                    .unwrap();
            }
        }

        Self {
            state,
            preferences: Preferences::default(),
            lsp_handle,
            modal_list_state: widgets::ListState::default(),
            rt,
        }
    }

    pub fn perform_action(&mut self, action: Action) {
        perform_action(
            action,
            &mut self.state,
            &mut self.preferences,
            &mut self.lsp_handle,
        );
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

                if let Some(message) = self.lsp_handle.recv_message_sync() {
                    self.state.update_view = true;
                    match message {
                        rift_core::lsp::client::IncomingMessage::Response(response) => {
                            if response.error.is_some() {
                                tracing::error!(
                                    "---Error: Message Id: {}\n\n{:#?}---\n",
                                    response.id,
                                    response.error.unwrap()
                                );
                            } else if self.lsp_handle.id_method[&response.id]
                                == "textDocument/hover"
                                && response.result.is_some()
                            {
                                let message = response.result.unwrap()["contents"]["value"]
                                    .as_str()
                                    .unwrap()
                                    .to_string();
                                // self.info_modal.info = message;
                                // self.info_modal.active = true;
                                // self.editor_focused = false;
                            } else if self.lsp_handle.id_method[&response.id]
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
                                                    row: item["textEdit"]["range"]["end"]["line"]
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
                                                    row: item["textEdit"]["range"]["start"]["line"]
                                                        .as_u64()
                                                        .unwrap()
                                                        as usize,
                                                    column: item["textEdit"]["range"]["start"]
                                                        ["character"]
                                                        .as_u64()
                                                        .unwrap()
                                                        as usize,
                                                },
                                            },
                                        },
                                    });
                                }
                                // self.completion_menu.set_items(completion_items);
                                // self.completion_menu.active = true;
                                // self.editor_focused = false;
                            } else if self.lsp_handle.id_method[&response.id]
                                == "textDocument/formatting"
                                && response.result.is_some()
                            {
                                let edits = response.result.unwrap().as_array().unwrap().clone();
                                for edit in edits {
                                    let text_edit = types::TextEdit {
                                        text: edit["newText"].as_str().unwrap().to_owned(),
                                        range: Selection {
                                            cursor: Cursor {
                                                row: edit["range"]["end"]["line"].as_u64().unwrap()
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
                                        &mut self.preferences,
                                        &mut self.lsp_handle,
                                    );
                                    perform_action(
                                        Action::InsertText(text_edit.text, text_edit.range.mark),
                                        &mut self.state,
                                        &mut self.preferences,
                                        &mut self.lsp_handle,
                                    );
                                }
                            } else {
                                let message = format!(
                                    "---Response to: {}({})\n\n{:#?}---\n",
                                    self.lsp_handle.id_method[&response.id],
                                    response.id,
                                    response.result
                                );
                                tracing::info!("{}", message);
                            }
                        }
                        rift_core::lsp::client::IncomingMessage::Notification(notification) => {
                            let message = format!(
                                "---Notification: {}\n\n{:#?}---\n",
                                notification.method, notification.params
                            );
                            tracing::info!("{}", message);
                            // self.diagnostics_overlay.info = message;
                        }
                    }
                }

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
                            match token.1 {
                                rift_core::buffer::instance::HighlightType::None => {
                                    style = style
                                        .fg(color_from_rgb(self.preferences.theme.highlight_none));
                                }
                                rift_core::buffer::instance::HighlightType::White => {
                                    style = style
                                        .fg(color_from_rgb(self.preferences.theme.highlight_white));
                                }
                                rift_core::buffer::instance::HighlightType::Red => {
                                    style = style
                                        .fg(color_from_rgb(self.preferences.theme.highlight_red));
                                }
                                rift_core::buffer::instance::HighlightType::Orange => {
                                    style = style.fg(color_from_rgb(
                                        self.preferences.theme.highlight_orange,
                                    ));
                                }
                                rift_core::buffer::instance::HighlightType::Blue => {
                                    style = style
                                        .fg(color_from_rgb(self.preferences.theme.highlight_blue));
                                }
                                rift_core::buffer::instance::HighlightType::Green => {
                                    style = style
                                        .fg(color_from_rgb(self.preferences.theme.highlight_green));
                                }
                                rift_core::buffer::instance::HighlightType::Purple => {
                                    style = style.fg(color_from_rgb(
                                        self.preferences.theme.highlight_purple,
                                    ));
                                }
                                rift_core::buffer::instance::HighlightType::Yellow => {
                                    style = style.fg(color_from_rgb(
                                        self.preferences.theme.highlight_yellow,
                                    ));
                                }
                                rift_core::buffer::instance::HighlightType::Gray => {
                                    style = style
                                        .fg(color_from_rgb(self.preferences.theme.highlight_gray));
                                }
                                rift_core::buffer::instance::HighlightType::Turquoise => {
                                    style = style.fg(color_from_rgb(
                                        self.preferences.theme.highlight_turquoise,
                                    ));
                                }
                            }
                            if token.2 {
                                style =
                                    style.bg(color_from_rgb(self.preferences.theme.selection_bg));
                            }
                            line_widget.push(text::Span::styled(&token.0, style));
                        }
                        lines.push(text::Line::from(line_widget));
                    }

                    frame.render_widget(text::Text::from(lines), h_layout[1]);

                    // Render cursor
                    let buf = frame.buffer_mut();
                    if let Some(cell) = buf.cell_mut((
                        self.state.relative_cursor.column as u16 + h_layout[1].x,
                        self.state.relative_cursor.row as u16 + h_layout[1].y,
                    )) {
                        if matches!(self.state.mode, Mode::Normal) {
                            cell.set_fg(color_from_rgb(
                                self.preferences.theme.cursor_normal_mode_fg,
                            ));
                            cell.set_bg(color_from_rgb(
                                self.preferences.theme.cursor_normal_mode_bg,
                            ));
                        } else {
                            cell.set_fg(color_from_rgb(
                                self.preferences.theme.cursor_insert_mode_fg,
                            ));
                            cell.set_bg(color_from_rgb(
                                self.preferences.theme.cursor_insert_mode_bg,
                            ));
                        }
                    }

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
                                        self.preferences.theme.gutter_text_current_line,
                                    )),
                                )
                                .alignment(ratatui::layout::Alignment::Right),
                            );
                        } else {
                            gutter_lines.push(
                                text::Line::styled(
                                    gutter_value,
                                    Style::new()
                                        .fg(color_from_rgb(self.preferences.theme.gutter_text)),
                                )
                                .alignment(ratatui::layout::Alignment::Right),
                            );
                        }
                    }
                    frame.render_widget(text::Text::from(gutter_lines), h_layout[0]);

                    // Render status line
                    let status = text::Line::from(vec![
                        format!(" {:#?} ", self.state.mode).into(),
                        format!(
                            " {} ",
                            self.state
                                .get_buffer_by_id(self.state.buffer_idx.unwrap())
                                .0
                                .file_path
                                .as_ref()
                                .unwrap()
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

                // Render Modal
                if self.state.modal_open {
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
                        .modal_options_filtered
                        .iter()
                        .map(|entry| entry.name.clone())
                        .collect::<widgets::List>();
                    frame.render_widget(widgets::Clear, popup_area);
                    frame.render_widget(modal_block, popup_area);
                    frame.render_widget(&self.state.modal_input, modal_layout[0]);
                    frame.render_stateful_widget(
                        modal_list,
                        modal_layout[2],
                        &mut self.modal_list_state,
                    );
                }
            })?;

            // Handle keyboard events
            if event::poll(Duration::from_millis(5))? {
                if let event::Event::Key(key) = event::read()? {
                    self.state.update_view = true;
                    if key.kind == KeyEventKind::Press {
                        if self.state.modal_open {
                            if let KeyCode::Char(char) = key.code {
                                self.state.modal_input.push(char);
                                self.state.modal_options_filtered = self
                                    .state
                                    .modal_options
                                    .iter()
                                    .filter(|entry| entry.path.starts_with(&self.state.modal_input))
                                    .cloned()
                                    .collect();
                            } else if key.code == KeyCode::Tab {
                                if !self.state.modal_options_filtered.is_empty() {
                                    if self.state.modal_selection_idx.is_none() {
                                        self.state.modal_selection_idx = Some(0);
                                        self.modal_list_state.select(Some(0));
                                    } else {
                                        self.state.modal_selection_idx =
                                            Some(self.state.modal_selection_idx.unwrap() + 1);
                                        self.modal_list_state.select(Some(
                                            self.state.modal_selection_idx.unwrap() + 1,
                                        ));
                                        if self.state.modal_selection_idx.unwrap()
                                            >= self.state.modal_options_filtered.len()
                                        {
                                            self.state.modal_selection_idx = Some(0);
                                            self.modal_list_state.select(Some(0));
                                        }
                                    }

                                    self.state.modal_input = self.state.modal_options_filtered
                                        [self.state.modal_selection_idx.unwrap()]
                                    .path
                                    .clone();
                                } else {
                                    self.state.modal_selection_idx = None;
                                    self.modal_list_state.select(None);
                                }
                            } else if key.code == KeyCode::Backspace {
                                self.state.modal_input.pop();
                                self.state.modal_options_filtered = self
                                    .state
                                    .modal_options
                                    .iter()
                                    .filter(|entry| entry.path.starts_with(&self.state.modal_input))
                                    .cloned()
                                    .collect();
                            } else if key.code == KeyCode::Enter {
                                if self.state.modal_selection_idx.is_some() {
                                    let entry = &self.state.modal_options_filtered
                                        [self.state.modal_selection_idx.unwrap()];
                                    if !entry.is_dir {
                                        let path = entry.path.clone();
                                        let initial_text =
                                            file_io::read_file_content(&path).unwrap();
                                        let buffer = LineBuffer::new(
                                            initial_text.clone(),
                                            Some(path.clone()),
                                        );
                                        self.state.buffer_idx = Some(self.state.add_buffer(buffer));
                                        self.state.modal_open = false;
                                        self.state.modal_options = vec![];
                                        self.state.modal_options_filtered = vec![];
                                        self.state.modal_selection_idx = None;
                                        self.modal_list_state.select(None);
                                        self.state.modal_input = "".into();

                                        self.lsp_handle
                                            .send_notification_sync(
                                                "textDocument/didOpen".to_string(),
                                                Some(LSPClientHandle::did_open_text_document(
                                                    path.clone(),
                                                    initial_text,
                                                )),
                                            )
                                            .unwrap();
                                    } else {
                                        self.state.modal_input = entry.path.clone();

                                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                                            self.state.workspace_folder = entry.path.clone();
                                            self.lsp_handle
                                                .init_lsp_sync(self.state.workspace_folder.clone());
                                        }

                                        #[cfg(target_os = "windows")]
                                        {
                                            self.state.modal_input.push('\\');
                                        }

                                        #[cfg(any(target_os = "linux", target_os = "macos"))]
                                        {
                                            self.state.modal_input.push('/');
                                        }

                                        self.state.modal_options =
                                            file_io::get_directory_entries(&entry.path).unwrap();
                                        self.state.modal_options_filtered =
                                            self.state.modal_options.clone();
                                        self.state.modal_selection_idx = None;
                                        self.modal_list_state.select(None);
                                    }
                                }
                            } else if key.code == KeyCode::Esc {
                                self.state.modal_open = false;
                                self.state.modal_options = vec![];
                                self.state.modal_options_filtered = vec![];
                                self.state.modal_selection_idx = None;
                                self.modal_list_state.select(None);
                                self.state.modal_input = "".into();
                            }
                        } else if matches!(self.state.mode, Mode::Normal) {
                            if key.code == KeyCode::Char('q') {
                                return Ok(());
                            } else if key.code == KeyCode::Char('i') {
                                self.perform_action(Action::EnterInsertMode);
                            } else if key.code == KeyCode::Char('f') {
                                self.perform_action(Action::OpenFile);
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
                            } else if key.code == KeyCode::Char('x') {
                                self.perform_action(Action::SelectCurrentLine);
                            } else if key.code == KeyCode::Char('X') {
                                self.perform_action(Action::SelectAndExtentCurrentLine);
                            } else if key.code == KeyCode::Char('w') {
                                self.perform_action(Action::SelectTillEndOfWord);
                            } else if key.code == KeyCode::Char('W') {
                                self.perform_action(Action::ExtendSelectTillEndOfWord);
                            } else if key.code == KeyCode::Char('b') {
                                self.perform_action(Action::SelectTillStartOfWord);
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
                            } else if key.code == KeyCode::Char('>') {
                                self.perform_action(Action::AddIndent);
                            } else if key.code == KeyCode::Char('<') {
                                self.perform_action(Action::RemoveIndent);
                            }
                        } else if matches!(self.state.mode, Mode::Insert) {
                            if key.code == KeyCode::Esc {
                                self.perform_action(Action::QuitInsertMode);
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

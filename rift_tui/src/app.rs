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
    buffer::line_buffer::LineBuffer,
    io::file_io,
    lsp::client::{start_lsp, LSPClientHandle},
    preferences::{Color, Preferences},
    state::{EditorState, Mode},
};

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
    pub fn new(rt: tokio::runtime::Runtime) -> Self {
        let lsp_handle = rt.block_on(async { start_lsp().await.unwrap() });
        Self {
            state: EditorState::default(),
            preferences: Preferences::default(),
            lsp_handle,
            modal_list_state: widgets::ListState::default(),
            rt,
        }
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
                        format!("{:#?}", self.state.mode).into(),
                        format!(
                            "{:#?}",
                            self.state
                                .get_buffer_by_id(self.state.buffer_idx.unwrap())
                                .1
                                .cursor
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

                                        if key.modifiers == KeyModifiers::SHIFT {
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
                                perform_action(
                                    Action::EnterInsertMode,
                                    &mut self.state,
                                    &mut self.preferences,
                                    &mut self.lsp_handle,
                                );
                            } else if key.code == KeyCode::Char('f') {
                                perform_action(
                                    Action::OpenFile,
                                    &mut self.state,
                                    &mut self.preferences,
                                    &mut self.lsp_handle,
                                );
                            }
                        } else if matches!(self.state.mode, Mode::Insert) {
                            if key.code == KeyCode::Esc {
                                perform_action(
                                    Action::QuitInsertMode,
                                    &mut self.state,
                                    &mut self.preferences,
                                    &mut self.lsp_handle,
                                );
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

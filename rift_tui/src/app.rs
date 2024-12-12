use std::time::Duration;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{self},
    DefaultTerminal,
};
use rift_core::{
    lsp::client::{start_lsp, LSPClientHandle},
    preferences::Preferences,
    state::EditorState,
};

pub struct App {
    pub state: EditorState,
    pub preferences: Preferences,
    pub lsp_handle: LSPClientHandle,
}

impl App {
    pub async fn new() -> Self {
        let lsp_handle = start_lsp().await.unwrap();
        Self {
            state: EditorState::default(),
            preferences: Preferences::default(),
            lsp_handle,
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| {
                // Layout
                let v_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Fill(1), Constraint::Length(1)])
                    .split(frame.area());
                let h_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(5), Constraint::Fill(1)])
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

                // Compute view if updated
                if self.state.update_view {
                    self.state.relative_cursor =
                        self.update_visible_lines(visible_lines, max_characters);
                    self.state.update_view = false;
                }

                // Render text
                for line in &self.state.highlighted_text {
                    for token in line {
                        // job.append(
                        //     &token.0,
                        //     egui::TextFormat {
                        //         color: match &token.1 {
                        //             HighlightType::None => {
                        //                 self.preferences.theme.highlight_none.into()
                        //             }
                        //             HighlightType::White => {
                        //                 self.preferences.theme.highlight_white.into()
                        //             }
                        //             HighlightType::Red => {
                        //                 self.preferences.theme.highlight_red.into()
                        //             }
                        //             HighlightType::Orange => {
                        //                 self.preferences.theme.highlight_orange.into()
                        //             }
                        //             HighlightType::Blue => {
                        //                 self.preferences.theme.highlight_blue.into()
                        //             }
                        //             HighlightType::Green => {
                        //                 self.preferences.theme.highlight_green.into()
                        //             }
                        //             HighlightType::Purple => {
                        //                 self.preferences.theme.highlight_purple.into()
                        //             }
                        //             HighlightType::Yellow => {
                        //                 self.preferences.theme.highlight_yellow.into()
                        //             }
                        //             HighlightType::Gray => {
                        //                 self.preferences.theme.highlight_gray.into()
                        //             }
                        //             HighlightType::Turquoise => {
                        //                 self.preferences.theme.highlight_turquoise.into()
                        //             }
                        //         },
                        //         background: match &token.2 {
                        //             true => self.preferences.theme.selection_bg.into(),
                        //             false => Color32::TRANSPARENT,
                        //         },
                        //     },
                        // )
                    }
                }

                let greeting = widgets::Paragraph::new(format!("{:#?}", h_layout[1]))
                    .white()
                    .on_dark_gray();
                frame.render_widget(greeting, h_layout[1]);

                // Render Modal
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
                frame.render_widget(widgets::Clear, popup_area);
                frame.render_widget(modal_block, popup_area);
                frame.render_widget("Hello", modal_layout[0]);
                frame.render_widget("Hello World", modal_layout[2]);
            })?;

            // Handle keyboard events
            if event::poll(Duration::from_millis(5))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        return Ok(());
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

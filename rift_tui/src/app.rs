use std::io::stdout;

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    style::Stylize,
    widgets::{Paragraph, Widget},
    Frame, Terminal,
};
use rift_explorer::buffer::line_buffer::{Language, LineTextBuffer, Selection};
use rift_generator::io::file_handling;

use crate::cli::CLI;

/// Type for ratatui terminal with crossterm backend
pub type Tui = Terminal<CrosstermBackend<std::io::Stdout>>;

/// State for each open buffer
pub struct EditorState {
    pub buffer: rift_explorer::buffer::line_buffer::LineTextBuffer,
    pub selection: rift_explorer::buffer::line_buffer::Selection,
    pub scroll_x: usize,
    pub scroll_y: usize,
}

/// Text Editor with State using ratatui for TUI
pub struct Editor {
    /// Exit flag
    pub exit: bool,
    /// Text buffers with extra state
    pub buffers: Vec<EditorState>,
    /// Current buffer index
    pub buffer_idx: usize,
    /// Previously pressed key
    pub prev_key_press: KeyCode,
    /// CLI arguments
    cli_args: CLI,
}

/// Initialize the TUI
pub fn init() -> std::io::Result<Tui> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal: Tui = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

/// Restore terminal to original state
pub fn restore() -> std::io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

impl Editor {
    /// Create a new instance of the editor
    pub fn new(cli_args: CLI) -> std::io::Result<Self> {
        Ok(Self {
            buffers: vec![],
            exit: false,
            prev_key_press: KeyCode::Null,
            cli_args,
            buffer_idx: 0,
        })
    }

    /// TUI main event loop
    pub fn run(&mut self, terminal: &mut Tui) -> std::io::Result<()> {
        self.create_scratchpad_buffer();
        while !self.exit {
            terminal.draw(|frame| {
                self.render_frame(frame);
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Immediate rendering
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// Handle events
    fn handle_events(&mut self) -> std::io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.exit = true,
                        KeyCode::Char('f') => {
                            if self.prev_key_press == KeyCode::Char(' ') {
                                self.open_file();
                            }
                        }
                        _ => {}
                    }
                    self.prev_key_press = key.code;
                }
            }
        }
        Ok(())
    }

    /// Open file as buffer
    fn open_file(&mut self) {
        let file_content = file_handling::read_file_content(&self.cli_args.path).unwrap();
        let mut editor = EditorState {
            buffer: LineTextBuffer::new(file_content),
            selection: Selection::default(),
            scroll_x: 0,
            scroll_y: 0,
        };
        editor.buffer.language = Language::Python;
        editor.buffer.highlight();
        self.buffers.push(editor);
        self.buffer_idx = self.buffers.len() - 1;
    }

    /// Open scratchpad buffer
    fn create_scratchpad_buffer(&mut self) {
        let mut editor = EditorState {
            buffer: LineTextBuffer::new("".to_owned()),
            selection: Selection::default(),
            scroll_x: 0,
            scroll_y: 0,
        };
        editor.buffer.language = Language::Python;
        editor.buffer.highlight();
        self.buffers.push(editor);
        self.buffer_idx = self.buffers.len() - 1;
    }
}

impl Widget for &Editor {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.left() + 5..area.right() - 5 {
            for y in area.top() + 5..area.bottom() - 5 {
                buf.get_mut(x, y).set_bg(Color::DarkGray);
            }
        }
        Paragraph::new(format!("{}", self.buffer_idx).green()).render(area, buf);
    }
}

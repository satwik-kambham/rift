use std::io::stdout;

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    style::Stylize,
    widgets::Paragraph,
    Terminal,
};

/// Type for ratatui terminal with crossterm backend
pub type Tui = Terminal<CrosstermBackend<std::io::Stdout>>;

/// Text Editor with State using ratatui for TUI
pub struct Editor {
    /// Main TUI handler
    pub tui: Tui,
    /// Exit flag
    pub exit: bool,
    /// Text buffers
    pub buffers: Vec<rift_explorer::buffer::line_buffer::LineTextBuffer>,
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
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            buffers: vec![],
            tui: init()?,
            exit: false,
        })
    }

    /// TUI main event loop
    pub fn render(mut self) -> std::io::Result<()> {
        while !self.exit {
            self.tui.draw(|frame| {
                let area = frame.size();
                frame.render_widget(Paragraph::new("sdfsdf".green()), area);
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        self.exit = true;
                    }
                }
            }
        }
        Ok(())
    }
}

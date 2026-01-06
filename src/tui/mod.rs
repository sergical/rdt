mod app;
mod event;
mod ui;

pub use app::App;

use crate::error::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;

/// Run the TUI application
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode().map_err(|e| crate::error::RdtError::Tui(e.to_string()))?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| crate::error::RdtError::Tui(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::error::RdtError::Tui(e.to_string()))?;

    // Create app and run
    let mut app = App::new();
    let result = app.run(&mut terminal).await;

    // Restore terminal
    disable_raw_mode().map_err(|e| crate::error::RdtError::Tui(e.to_string()))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(|e| crate::error::RdtError::Tui(e.to_string()))?;
    terminal.show_cursor()
        .map_err(|e| crate::error::RdtError::Tui(e.to_string()))?;

    result
}

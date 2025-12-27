pub mod app;
pub mod event;
pub mod input;
pub mod ui;
pub mod widgets;

use std::path::PathBuf;
use std::sync::Arc;

use crate::soulseek::SoulSeekClientContext;
use color_eyre::Result;

/// Main entry point for the TUI
pub async fn run(
    soulseek_context: Arc<SoulSeekClientContext>,
    download_output_directory: PathBuf,
) -> Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::prelude::*;
    use std::io;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run app
    let mut app = app::App::new(soulseek_context, download_output_directory);
    let result = app.run(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

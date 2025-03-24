mod kanban;
mod ops;

use kanban::{App, run_app};
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

fn main() -> Result<(), io::Error> {
    // Check for KANBAN_DIR environment variable
    if std::env::var("KANBAN_DIR").is_err() {
        eprintln!("Warning: KANBAN_DIR environment variable not set.");
        eprintln!("Changes won't be saved. Set KANBAN_DIR to enable persistence.");
        // We continue anyway, the app will work but without persistence
    }

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app - it will start in board selection mode automatically
    let app = App::new("Kanban Board");
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

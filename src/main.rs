#![allow(dead_code)]
mod ui;
mod types;
mod input;

use std::{io};
use crossterm::{event::*, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use crate::input::handle_input;
use crate::types::*;

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(Project::load());

    loop {
        terminal.draw(|f| ui::draw(f, &mut state))?;
        handle_input(&mut state)?;
        if state.quit { break }
    }

    // restore terminal
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

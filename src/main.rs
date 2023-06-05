#![allow(unused_imports)]
#![allow(dead_code)]
use kanban_tui::*;
use crossterm::{
    event::*,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(Project::load()?);

    while !state.quit {
        terminal.draw(|f| kanban_tui::draw(f, &mut state))?;
        kanban_tui::handle_input(&mut state)?;
    }

    state.project.save();

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

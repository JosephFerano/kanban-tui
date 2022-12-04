use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crate::types::{AppState};

pub fn handle_input(state: &mut AppState) -> Result<(), std::io::Error> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => state.quit = true,
            KeyCode::Char('h') |
            KeyCode::Left      => state.select_previous_column(),
            KeyCode::Char('j') |
            KeyCode::Down      => state.select_next_task(),
            KeyCode::Char('k') |
            KeyCode::Up        => state.select_previous_task(),
            KeyCode::Char('l') |
            KeyCode::Right     => state.select_next_column(),
            _ => {}
        }
    }
    Ok(())
}

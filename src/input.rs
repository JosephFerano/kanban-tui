use std::cmp::min;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crate::types::AppState;

pub fn handle_input(state: &mut AppState) -> Result<(), std::io::Error> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => state.quit = true,
            KeyCode::Char('h') |
            KeyCode::Left      => state.selected_column = state.selected_column.saturating_sub(1),
            KeyCode::Char('j') |
            KeyCode::Down      => state.selected_task[state.selected_column] += 1,
            KeyCode::Char('k') |
            KeyCode::Up        => state.selected_task[state.selected_column] -= 1,
            KeyCode::Char('l') |
            KeyCode::Right     => state.selected_column = min(state.selected_column + 1, 4),
            _ => {}
        }
    }
    Ok(())
}

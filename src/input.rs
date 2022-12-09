use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crate::app::{AppState};

pub fn handle_input(state: &mut AppState) -> Result<(), std::io::Error> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => state.quit = true,
            KeyCode::Char('h') |
            KeyCode::Left      => { state.select_previous_column(); },
            KeyCode::Char('j') |
            KeyCode::Down      => state.select_next_task(),
            KeyCode::Char('k') |
            KeyCode::Up        => state.select_previous_task(),
            KeyCode::Char('l') |
            KeyCode::Right     => { state.select_next_column(); },
            KeyCode::Char('<') |
            KeyCode::Char('H') => state.move_task_previous_column(),
            KeyCode::Char('>') |
            KeyCode::Char('L') => state.move_task_next_column(),
            KeyCode::Char('=') |
            KeyCode::Char('J') => state.move_task_down(),
            KeyCode::Char('-') |
            KeyCode::Char('K') => state.move_task_up(),
            KeyCode::Char('p') => {
                match state.popup_text {
                    None => state.popup_text = Some("".to_string()),
                    Some(_) => state.popup_text = None,
                }
            }
            _ => {}
        }
    }
    Ok(())
}

use std::cmp::min;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use int_enum::IntEnum;
use crate::types::{AppState, TaskStatus};

pub fn handle_input(state: &mut AppState) -> Result<(), std::io::Error> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => state.quit = true,
            KeyCode::Char('h') |
            KeyCode::Left      => state.selected_column = state.selected_column.saturating_sub(1),
            KeyCode::Char('j') |
            KeyCode::Down => {
                let column: TaskStatus = TaskStatus::from_int(state.selected_column).unwrap();
                let tasks = state.current_project.tasks_per_column.get(&column).unwrap();
                if tasks.len() > 0 {
                    let mins = min(state.selected_task[state.selected_column] + 1, tasks.len() - 1);
                    state.selected_task[state.selected_column] = mins;
                }
            }
            KeyCode::Char('k') |
            KeyCode::Up        => state.selected_task[state.selected_column] = state.selected_task[state.selected_column].saturating_sub(1),
            KeyCode::Char('l') |
            KeyCode::Right     => state.selected_column = min(state.selected_column + 1, 4),
            _ => {}
        }
    }
    Ok(())
}

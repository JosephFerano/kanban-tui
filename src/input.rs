use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crate::app::{NewTask, AppState};

pub fn handle_input(state: &mut AppState) -> Result<(), std::io::Error> {
    let project = &mut state.project;
    let column = project.get_selected_column_mut();
    if let Event::Key(key) = event::read()? {
        match key.code {
            // KeyCode::BackTab   =>
            // KeyCode::Tab       =>
            KeyCode::Char('q') => state.quit = true,
            KeyCode::Char('h') |
            KeyCode::Left      => { project.select_previous_column(); },
            KeyCode::Char('j') |
            KeyCode::Down      => column.select_next_task(),
            KeyCode::Char('k') |
            KeyCode::Up        => column.select_previous_task(),
            KeyCode::Char('l') |
            KeyCode::Right     => { project.select_next_column(); },
            KeyCode::Char('<') |
            KeyCode::Char('H') => project.move_task_previous_column(),
            KeyCode::Char('>') |
            KeyCode::Char('L') => project.move_task_next_column(),
            KeyCode::Char('=') |
            KeyCode::Char('J') => project.move_task_down(),
            KeyCode::Char('-') |
            KeyCode::Char('K') => project.move_task_up(),
            KeyCode::Char('p') => {
                match state.new_task_state {
                    None => state.new_task_state = Some(NewTask::default()),
                    Some(_) => state.new_task_state = None,
                }
            }
            _ => {}
        }
    }
    Ok(())
}

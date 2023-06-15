use crate::app::{State, TaskEditFocus, TaskState};
use crossterm::event;
use crossterm::event::{Event, KeyCode};

pub fn handle_task_edit(state: &mut State<'_>, key: event::KeyEvent) -> Result<(), anyhow::Error> {
    if let Some(mut task) = state.task_edit_state.take() {
        let mut clear_task = false;
        match task.focus {
            // TODO: Handle wrapping around the enum rather than doing it manually
            TaskEditFocus::Title => match key.code {
                KeyCode::Tab => task.focus = TaskEditFocus::Description,
                KeyCode::BackTab => task.focus = TaskEditFocus::CancelBtn,
                KeyCode::Enter => (),
                _ => {
                    task.title.input(key);
                }
            },
            TaskEditFocus::Description => match key.code {
                KeyCode::Tab => task.focus = TaskEditFocus::ConfirmBtn,
                KeyCode::BackTab => task.focus = TaskEditFocus::Title,
                _ => {
                    task.description.input(key);
                }
            },
            TaskEditFocus::ConfirmBtn => match key.code {
                KeyCode::Tab => task.focus = TaskEditFocus::CancelBtn,
                KeyCode::BackTab => task.focus = TaskEditFocus::Description,
                KeyCode::Enter => {
                    let title = task.title.clone().into_lines().join("\n");
                    let description = task.description.clone().into_lines().join("\n");
                    if task.is_edit {
                        state.edit_task(title, description)?;
                    } else {
                        state.add_new_task(title, description)?;
                    }
                    clear_task = true;
                }
                _ => (),
            },
            TaskEditFocus::CancelBtn => match key.code {
                KeyCode::Tab => task.focus = TaskEditFocus::Title,
                KeyCode::BackTab => task.focus = TaskEditFocus::ConfirmBtn,
                KeyCode::Enter => {
                    clear_task = true;
                }
                _ => (),
            },
        }
        if !clear_task {
            state.task_edit_state = Some(task);
        }
    }
    Ok(())
}

pub fn handle_main(state: &mut State<'_>, key: event::KeyEvent) -> Result<(), anyhow::Error> {
    match key.code {
        KeyCode::Char('q') => state.quit = true,
        KeyCode::Char('h') | KeyCode::Left => {
            state.select_previous_column()?;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            state.select_next_task()?;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.select_previous_task()?;
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.select_next_column()?;
        }
        KeyCode::Char('g') => {
            state.select_first_task()?;
        }
        KeyCode::Char('G') => {
            state.select_last_task()?;
        }
        KeyCode::Char('H') => {
            state.move_task_previous_column()?;
        }
        KeyCode::Char('L') => {
            state.move_task_next_column()?;
        }
        KeyCode::Char('J') => {
            state.move_task_down()?;
        }
        KeyCode::Char('K') => {
            state.move_task_up()?;
        }
        KeyCode::Char('n') => state.task_edit_state = Some(TaskState::default()),
        KeyCode::Char('e') => state.task_edit_state = state.get_task_state_from_current(),
        KeyCode::Char('D') => {
            state.delete_task()?;
        }
        _ => {}
    }
    Ok(())
}

/// # Errors
///
/// Crossterm `event::read()` might return an error
pub fn handle(state: &mut State<'_>) -> Result<(), anyhow::Error> {
    if let Event::Key(key) = event::read()? {
        if state.task_edit_state.is_some() {
            handle_task_edit(state, key)?;
        } else {
            handle_main(state, key)?;
        }
    }
    Ok(())
}

use crate::app::{State, TaskEditFocus, TaskState};
use crossterm::event;
use crossterm::event::{Event, KeyCode};

// pub fn handle_task_edit(state: &mut State<'_>, key: event::KeyEvent, mut task: TaskState<'_>) {
pub fn handle_task_edit(
    state: &mut State<'_>,
    key: event::KeyEvent,
) {
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
                        let column = state.get_selected_column_mut();
                        if let Some(selected_task) = column.get_selected_task_mut() {
                            selected_task.title = title;
                            selected_task.description = description;
                            let cloned = selected_task.clone();
                            state.db_conn.update_task_text(&cloned);
                        }
                    } else {
                        let col_id = state.get_selected_column().id;
                        let selected_task_idx = state.get_selected_column().selected_task_idx;
                        let task = state.db_conn.insert_new_task(title, description, col_id);
                        state.get_selected_column_mut().add_task(task);
                        state.db_conn.set_selected_task_for_column(selected_task_idx, col_id);
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
}

pub fn handle_main(state: &mut State<'_>, key: event::KeyEvent) {
    let column = state.get_selected_column_mut();
    match key.code {
        KeyCode::Char('q') => state.quit = true,
        KeyCode::Char('h') | KeyCode::Left => {
            state.select_previous_column();
            state.db_conn.set_selected_column(state.selected_column_idx);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            column.select_next_task();
            let task_idx = column.selected_task_idx;
            let col_id = state.get_selected_column().id;
            state.db_conn.set_selected_task_for_column(task_idx, col_id);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            column.select_previous_task();
            let task_idx = column.selected_task_idx;
            let col_id = state.get_selected_column().id;
            state.db_conn.set_selected_task_for_column(task_idx, col_id);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.select_next_column();
            state.db_conn.set_selected_column(state.selected_column_idx);
        }
        KeyCode::Char('g') => {
            column.select_first_task();
            let task_idx = column.selected_task_idx;
            let col_id = state.get_selected_column().id;
            state.db_conn.set_selected_task_for_column(task_idx, col_id);
        }
        KeyCode::Char('G') => {
            column.select_last_task();
            let task_idx = column.selected_task_idx;
            let col_id = state.get_selected_column().id;
            state.db_conn.set_selected_task_for_column(task_idx, col_id);
        }
        KeyCode::Char('H') => {
            if !column.tasks.is_empty() {
                state.move_task_previous_column();
            }
        }
        KeyCode::Char('L') => {
            if !column.tasks.is_empty() {
                state.move_task_next_column();
            }
        }
        KeyCode::Char('J') => { state.move_task_down(); },
        KeyCode::Char('K') => { state.move_task_up(); },
        KeyCode::Char('n') => state.task_edit_state = Some(TaskState::default()),
        KeyCode::Char('e') => state.task_edit_state = column.get_task_state_from_current(),
        KeyCode::Char('D') => {
            if !column.tasks.is_empty() {
                state.delete_task();
            }
        }
        _ => {}
    }
}

/// # Errors
///
/// Crossterm `event::read()` might return an error
///
/// # Panics
///
/// Shouldn't really panic because there are checks to ensure we can unwrap safely
pub fn handle(state: &mut State<'_>) -> Result<(), std::io::Error> {
    if let Event::Key(key) = event::read()? {
        if state.task_edit_state.is_some() {
            handle_task_edit(state, key);
        } else {
            handle_main(state, key);
        }
    }
    Ok(())
}

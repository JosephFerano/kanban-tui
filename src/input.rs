use crate::app::{State, TaskEditFocus, TaskState};
use crate::db;
use crossterm::event;
use crossterm::event::{Event, KeyCode};

pub fn handle_task_edit(state: &mut State<'_>, key: event::KeyEvent, mut task: TaskState<'_>) {
    let project = &mut state.project;
    let column = project.get_selected_column_mut();
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
                    if let Some(selected_task) = column.get_selected_task_mut() {
                        selected_task.title = title;
                        selected_task.description = description;
                        db::update_task_text(&state.db_conn, selected_task);
                    }
                } else {
                    let task = db::insert_new_task(&state.db_conn, title, description, column);
                    column.add_task(task);
                }
                state.task_edit_state = None;
            }
            _ => (),
        },
        TaskEditFocus::CancelBtn => match key.code {
            KeyCode::Tab => task.focus = TaskEditFocus::Title,
            KeyCode::BackTab => task.focus = TaskEditFocus::ConfirmBtn,
            KeyCode::Enter => {
                state.task_edit_state = None;
            }
            _ => (),
        },
    };
}

pub fn handle_main(state: &mut State<'_>, key: event::KeyEvent) {
    let project = &mut state.project;
    let column = project.get_selected_column_mut();
    match key.code {
        KeyCode::Char('q') => state.quit = true,
        KeyCode::Char('h') | KeyCode::Left => {
            project.select_previous_column();
        }
        KeyCode::Char('j') | KeyCode::Down => column.select_next_task(),
        KeyCode::Char('k') | KeyCode::Up => column.select_previous_task(),
        KeyCode::Char('l') | KeyCode::Right => {
            project.select_next_column();
        }
        KeyCode::Char('g') => column.select_first_task(),
        KeyCode::Char('G') => column.select_last_task(),
        KeyCode::Char('H') => {
            if !column.tasks.is_empty() {
                project.move_task_previous_column();
                let col = project.get_selected_column();
                let t = col.get_selected_task().unwrap();
                db::move_task_to_column(&state.db_conn, t, col);
            }
        }
        KeyCode::Char('L') => {
            if !column.tasks.is_empty() {
                project.move_task_next_column();
                let col = project.get_selected_column();
                let t = col.get_selected_task().unwrap();
                db::move_task_to_column(&state.db_conn, t, col);
            }
        }
        KeyCode::Char('J') => {
            if column.move_task_down() {
                let task1 = column.get_selected_task().unwrap();
                let task2 = column.get_previous_task().unwrap();
                db::swap_task_order(&mut state.db_conn, task1, task2);
            }
        }
        KeyCode::Char('K') => {
            if column.move_task_up() {
                let task1 = column.get_selected_task().unwrap();
                let task2 = column.get_next_task().unwrap();
                db::swap_task_order(&mut state.db_conn, task1, task2);
            }
        }
        KeyCode::Char('n') => state.task_edit_state = Some(TaskState::default()),
        KeyCode::Char('e') => {
            state.task_edit_state = column.get_task_state_from_curr_selected_task();
        }
        KeyCode::Char('D') => {
            if !column.tasks.is_empty() {
                db::delete_task(&state.db_conn, column.get_selected_task().unwrap());
                column.remove_task();
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
        if let Some(task) = state.task_edit_state.take() {
            handle_task_edit(state, key, task);
        } else {
            handle_main(state, key);
        }
    }
    Ok(())
}

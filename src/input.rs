use crate::app::{State, TaskEditFocus, TaskState, EDIT_WINDOW_FOCUS_STATES};
use anyhow::Error;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use int_enum::IntEnum;

pub fn cycle_focus(task: &mut TaskState<'_>, forward: bool) -> Result<(), Error> {
    let cycle;
    if forward {
        cycle = (task.focus.int_value() + 1) % EDIT_WINDOW_FOCUS_STATES;
    } else {
        cycle = (task.focus.int_value() - 1) % EDIT_WINDOW_FOCUS_STATES;
    }
    task.focus = TaskEditFocus::from_int(cycle)?;
    Ok(())
}

pub fn handle_task_edit(state: &mut State<'_>, key: event::KeyEvent) -> Result<(), Error> {
    // .take() the option so we can avoid borrow checker issues when
    // we try to edit the task since that mutably borrows State, then
    // assign later to task_edit_state
    let updated_task = if let Some(mut task) = state.task_edit_state.take() {
        match (key.code, task.focus) {
            (KeyCode::Tab, _) => {
                cycle_focus(&mut task, true)?;
                Some(task)
            }
            (KeyCode::BackTab, _) => {
                cycle_focus(&mut task, false)?;
                Some(task)
            }
            (KeyCode::Enter, TaskEditFocus::ConfirmBtn) => {
                // The structure of this function is so we avoid an
                // unncessary clone() here. We can just transfer
                // ownership of these strings right into the task
                // that's going to be created/updated
                let title = task.title.into_lines().join("\n");
                let description = task.description.into_lines().join("\n");
                if task.is_edit {
                    state.edit_task(title, description)?;
                } else {
                    state.add_new_task(title, description)?;
                }
                None
            }
            (KeyCode::Enter, TaskEditFocus::CancelBtn) => None,
            // Ignore enter on the title bar to effectively make it single line
            (KeyCode::Enter, TaskEditFocus::Title) => Some(task),
            (_, TaskEditFocus::Title) => {
                task.title.input(key);
                Some(task)
            }
            (_, TaskEditFocus::Description) => {
                task.description.input(key);
                Some(task)
            }
            _ => Some(task),
        }
    } else {
        None
    };
    state.task_edit_state = updated_task;
    Ok(())
}

pub fn handle_main(state: &mut State<'_>, key: event::KeyEvent) -> Result<(), Error> {
    match key.code {
        KeyCode::Char('q') => Ok(state.quit = true),
        KeyCode::Char('h') | KeyCode::Left => state.select_previous_column(),
        KeyCode::Char('j') | KeyCode::Down => state.select_next_task(),
        KeyCode::Char('k') | KeyCode::Up => state.select_previous_task(),
        KeyCode::Char('l') | KeyCode::Right => state.select_next_column(),
        KeyCode::Char('g') => state.select_first_task(),
        KeyCode::Char('G') => state.select_last_task(),
        KeyCode::Char('H') => state.move_task_previous_column(),
        KeyCode::Char('L') => state.move_task_next_column(),
        KeyCode::Char('J') => state.move_task_down(),
        KeyCode::Char('K') => state.move_task_up(),
        KeyCode::Char('n') => Ok(state.task_edit_state = Some(TaskState::default())),
        KeyCode::Char('e') => Ok(state.task_edit_state = state.get_task_state_from_current()),
        KeyCode::Char('D') => state.delete_task(),
        _ => Ok(()),
    }
}

/// # Errors
///
/// Crossterm `event::read()` might return an error
pub fn handle(state: &mut State<'_>) -> Result<(), Error> {
    if let Event::Key(key) = event::read()? {
        if state.task_edit_state.is_some() {
            handle_task_edit(state, key)?;
        } else {
            handle_main(state, key)?;
        }
    }
    Ok(())
}

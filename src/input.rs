use crate::app::{State, TaskEditFocus, TaskState};
use crate::{db};
use crossterm::event;
use crossterm::event::{Event, KeyCode};

/// # Errors
///
/// Crossterm `event::read()` might return an error
pub fn handle(state: &mut State<'_>) -> Result<(), std::io::Error> {
    let project = &mut state.project;
    let column = project.get_selected_column_mut();
    if let Event::Key(key) = event::read()? {
        match &mut state.task_edit_state {
            Some(task) => {
                // TODO: Extract this code to a separate function to avoid nesting
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
                                }
                            } else {
                                let task = db::insert_new_task(&state.db_conn, title, description, &column);
                                column.add_task(task);
                            }
                            state.task_edit_state = None;
                            project.save();
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
            None => match key.code {
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
                KeyCode::Char('H') => project.move_task_previous_column(),
                KeyCode::Char('L') => project.move_task_next_column(),
                KeyCode::Char('J') => project.move_task_down(),
                KeyCode::Char('K') => project.move_task_up(),
                KeyCode::Char('n') => state.task_edit_state = Some(TaskState::default()),
                KeyCode::Char('e') => {
                    state.task_edit_state = column.get_task_state_from_curr_selected_task()
                }
                KeyCode::Char('D') => {
                    column.remove_task();
                    db::delete_task(&state.db_conn, column.get_selected_task().unwrap());
                    // project.save();
                }
                _ => {}
            },
        }
    }
    Ok(())
}

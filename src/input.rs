#![allow(unused_imports)]
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crate::app::{TaskState, AppState, TaskEditFocus};
use std::io::{stdout, Write};
use tui_textarea::TextArea;

pub fn handle_input(state: &mut AppState) -> Result<(), std::io::Error> {
    let project = &mut state.project;
    let column = project.get_selected_column_mut();
    if let Event::Key(key) = event::read()? {
        match &mut state.task_edit_state {
            Some(task) => {
                // TODO: Extract this code to a separate function to avoid nesting
                match task.focus {
                    // TODO: Handle wrapping around the enum rather than doing it manually
                    TaskEditFocus::Title => {
                        match key.code {
                            KeyCode::Tab => task.focus = TaskEditFocus::Description,
                            KeyCode::BackTab => task.focus = TaskEditFocus::CancelBtn,
                            KeyCode::Enter => (),
                            _ => { task.title.input(key); }
                        }
                    }
                    TaskEditFocus::Description => {
                        match key.code {
                            KeyCode::Tab => task.focus = TaskEditFocus::CreateBtn,
                            KeyCode::BackTab => task.focus = TaskEditFocus::Title,
                            _ => { task.description.input(key); }
                        }
                    }
                    TaskEditFocus::CreateBtn => {
                        match key.code {
                            KeyCode::Tab => task.focus = TaskEditFocus::CancelBtn,
                            KeyCode::BackTab => task.focus = TaskEditFocus::Description,
                            KeyCode::Enter => {
                                let title = task.title.clone().into_lines().join("\n");
                                let description = task.description.clone().into_lines().clone().join("\n");
                                column.add_task(title, description);
                                state.task_edit_state = None
                            }
                            _ => (),
                        }
                    }
                    TaskEditFocus::CancelBtn => {
                        match key.code {
                            KeyCode::Tab => task.focus = TaskEditFocus::Title,
                            KeyCode::BackTab => task.focus = TaskEditFocus::CreateBtn,
                            KeyCode::Enter => {
                                state.task_edit_state = None
                            }
                            _ => (),
                        }
                    }
                };
            }
            None => {
                match key.code {
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
                    KeyCode::Char('n') => state.task_edit_state = Some(TaskState::default()),
                    KeyCode::Char('e') => {
                        match state.task_edit_state {
                            None => state.task_edit_state = Some(TaskState::default()),
                            Some(_) => state.task_edit_state = None,
                        }
                    }
                    KeyCode::Char('D') => column.remove_task(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

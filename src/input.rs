#![allow(unused_imports)]
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crate::app::{NewTask, AppState, NewTaskFocus};
use std::io::{stdout, Write};
use tui_textarea::TextArea;

pub fn handle_input(state: &mut AppState) -> Result<(), std::io::Error> {
    let project = &mut state.project;
    let column = project.get_selected_column_mut();
    if let Event::Key(key) = event::read()? {
        match &mut state.new_task_state {
            Some(task) => {
                match task.focus {
                    // TODO: Handle wrapping around the enum rather than doing it manually
                    NewTaskFocus::Title => {
                        match key.code {
                            KeyCode::Tab => task.focus = NewTaskFocus::Description,
                            KeyCode::BackTab => task.focus = NewTaskFocus::Buttons,
                            _ => { task.title.input(key); }
                        }
                    }
                    NewTaskFocus::Description => {
                        match key.code {
                            KeyCode::Tab => task.focus = NewTaskFocus::Buttons,
                            KeyCode::BackTab => task.focus = NewTaskFocus::Title,
                            _ => { task.description.input(key); }
                        }
                    }
                    
                    NewTaskFocus::Buttons => {
                        match key.code {
                            KeyCode::Tab => task.focus = NewTaskFocus::Title,
                            KeyCode::BackTab => task.focus = NewTaskFocus::Description,
                            KeyCode::Enter => {
                                // TODO: Need a function that clears state and adds a new TODO
                                // into the right column
                                state.new_task_state = None
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
                    KeyCode::Char('p') => {
                        match state.new_task_state {
                            None => state.new_task_state = Some(NewTask::default()),
                            Some(_) => state.new_task_state = None,
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// use indexmap::IndexMap;
// use int_enum::IntEnum;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use tui_textarea::TextArea;

use crate::db;

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub id: i64,
    pub name: String,
    pub selected_task_idx: usize,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
}

#[derive(Debug)]
pub enum TaskEditFocus {
    Title,
    Description,
    ConfirmBtn,
    CancelBtn,
}

pub struct TaskState<'a> {
    pub title: TextArea<'a>,
    pub description: TextArea<'a>,
    pub focus: TaskEditFocus,
    pub is_edit: bool,
}

impl Default for TaskState<'_> {
    fn default() -> Self {
        TaskState {
            title: TextArea::default(),
            description: TextArea::default(),
            focus: TaskEditFocus::Title,
            is_edit: false,
        }
    }
}

pub struct State<'a> {
    pub selected_column_idx: usize,
    pub columns: Vec<Column>,
    pub db_conn: db::DBConn,
    pub quit: bool,
    pub task_edit_state: Option<TaskState<'a>>,
}

impl<'a> State<'a> {
    /// Creates a new [`State`].
    ///
    /// # Panics
    ///
    /// Panics if we can't get all the columns from the database
    #[must_use]
    pub fn new(conn: Connection) -> Self {
        let db_conn = db::DBConn::new(conn);
        let columns = db_conn.get_all_columns().unwrap();
        let selected_column = db_conn.get_selected_column();
        State {
            columns,
            selected_column_idx: selected_column,
            quit: false,
            task_edit_state: None,
            db_conn,
        }
    }

    #[must_use]
    pub fn get_selected_column(&self) -> &Column {
        &self.columns[self.selected_column_idx]
    }

    pub fn get_selected_column_mut(&mut self) -> &mut Column {
        &mut self.columns[self.selected_column_idx]
    }

    pub fn select_previous_column(&mut self) {
        self.selected_column_idx = self.selected_column_idx.saturating_sub(1);
        self.db_conn.set_selected_column(self.selected_column_idx);
    }

    pub fn select_next_column(&mut self) {
        self.selected_column_idx = min(self.selected_column_idx + 1, self.columns.len() - 1);
        self.db_conn.set_selected_column(self.selected_column_idx);
    }

    #[must_use]
    pub fn get_selected_task(&self) -> Option<&Task> {
        let column = self.get_selected_column();
        column.tasks.get(column.selected_task_idx)
    }

    #[must_use]
    pub fn get_previous_task(&self) -> Option<&Task> {
        let column = self.get_selected_column();
        if column.selected_task_idx > 0 {
            column.tasks.get(column.selected_task_idx - 1)
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_next_task(&self) -> Option<&Task> {
        let column = self.get_selected_column();
        column.tasks.get(column.selected_task_idx + 1)
    }

    pub fn get_selected_task_mut(&mut self) -> Option<&mut Task> {
        let column = self.get_selected_column_mut();
        column.tasks.get_mut(column.selected_task_idx)
    }

    pub fn select_previous_task(&mut self) {
        let column = self.get_selected_column_mut();
        column.selected_task_idx = column.selected_task_idx.saturating_sub(1);

        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        self.db_conn.set_selected_task_for_column(task_idx, col_id);
    }

    pub fn select_next_task(&mut self) {
        let column = self.get_selected_column_mut();
        column.selected_task_idx = min(
            column.selected_task_idx + 1,
            column.tasks.len().saturating_sub(1),
        );

        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        self.db_conn.set_selected_task_for_column(task_idx, col_id);
    }

    pub fn select_first_task(&mut self) {
        let column = self.get_selected_column_mut();
        column.selected_task_idx = 0;

        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        self.db_conn.set_selected_task_for_column(task_idx, col_id);
    }

    pub fn select_last_task(&mut self) {
        let column = self.get_selected_column_mut();
        column.selected_task_idx = column.tasks.len().saturating_sub(1);

        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        self.db_conn.set_selected_task_for_column(task_idx, col_id);
    }

    #[must_use]
    pub fn get_task_state_from_current(&self) -> Option<TaskState<'a>> {
        self.get_selected_task().map(|t| TaskState {
            title: TextArea::from(t.title.lines()),
            description: TextArea::from(t.description.lines()),
            focus: TaskEditFocus::Title,
            is_edit: true,
        })
    }

    pub fn move_task_up(&mut self) {
        self.move_task(false);
    }

    pub fn move_task_down(&mut self) {
        self.move_task(true);
    }
    /// Returns the move task down of this [`State`].
    ///
    /// # Panics
    ///
    /// We have conditions to ensure this doesn't panic but we still unwrap()
    pub fn move_task(&mut self, is_down: bool) {
        let other_task = if is_down {
            self.get_next_task()
        } else {
            self.get_previous_task()
        };
        if let (Some(task1), Some(task2)) = (self.get_selected_task(), other_task) {
            let t1_id = task1.id;
            let t2_id = task2.id;
            let column = self.get_selected_column_mut();
            let task_idx = column.selected_task_idx;

            let other_idx = if is_down { task_idx + 1 } else { task_idx - 1 };
            column.tasks.swap(task_idx, other_idx);
            if is_down {
                column.selected_task_idx += 1;
            } else {
                column.selected_task_idx -= 1;
            }

            let col_id = column.id;
            self.db_conn.swap_task_order(t2_id, t1_id);
            self.db_conn.set_selected_task_for_column(task_idx, col_id);
        }
    }

    pub fn move_task_previous_column(&mut self) {
        self.move_task_to_column(false);
    }

    pub fn move_task_next_column(&mut self) {
        self.move_task_to_column(true);
    }

    fn move_task_to_column(&mut self, move_right: bool) {
        let can_move_right = move_right && self.selected_column_idx < self.columns.len() - 1;
        let can_move_left = !move_right && self.selected_column_idx > 0;

        let first_col = self.get_selected_column_mut();
        if first_col.tasks.is_empty() || !can_move_right && !can_move_left {
            return;
        }
        let t = first_col.tasks.remove(first_col.selected_task_idx);

        // Only move it if it was the last task
        if first_col.selected_task_idx == first_col.tasks.len() {
            self.select_previous_task();    
        }

        if move_right {
            self.select_next_column();
        } else {
            self.select_previous_column();
        }

        let col = self.get_selected_column_mut();
        col.tasks.push(t);
        self.select_last_task();
        if let Some(task) = self.get_selected_task() {
            self.db_conn.move_task_to_column(task, self.get_selected_column());
            self.db_conn.set_selected_column(self.selected_column_idx);
        }
    }

    pub fn add_new_task(&mut self, title: String, description: String) {
        let col_id = self.get_selected_column().id;
        let task = self.db_conn.insert_new_task(title, description, col_id);

        self.select_last_task();
        let selected_task_idx = self.get_selected_column().selected_task_idx;
        self.db_conn.set_selected_task_for_column(selected_task_idx, col_id);

        self.get_selected_column_mut().tasks.push(task);
    }

    pub fn edit_task(&mut self, title: String, description: String) {
        if let Some(selected_task) = self.get_selected_task_mut() {
            selected_task.title = title;
            selected_task.description = description;
            let cloned = selected_task.clone();
            self.db_conn.update_task_text(&cloned);
        }
    }

    /// Returns the delete task of this [`State`].
    ///
    /// # Panics
    ///
    /// We have conditions to ensure this doesn't panic but we still unwrap()
    pub fn delete_task(&mut self) {
        let column = self.get_selected_column();
        if column.tasks.is_empty() {
            return;
        }
        let task_id = self.get_selected_task().unwrap().id;
        let column = self.get_selected_column_mut();
        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        column.tasks.remove(column.selected_task_idx);
        self.select_next_task();
        self.db_conn.delete_task(task_id);
        self.db_conn.set_selected_task_for_column(task_idx, col_id);
    }
}

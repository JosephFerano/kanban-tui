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

impl State<'_> {
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

    pub fn select_previous_column(&mut self) -> &Column {
        self.selected_column_idx = self.selected_column_idx.saturating_sub(1);
        &self.columns[self.selected_column_idx]
    }

    pub fn select_next_column(&mut self) -> &Column {
        self.selected_column_idx = min(self.selected_column_idx + 1, self.columns.len() - 1);
        &self.columns[self.selected_column_idx]
    }

    fn move_task_to_column(&mut self, move_next: bool) {
        let col_idx = self.selected_column_idx;
        let cols_len = self.columns.len();
        let column = self.get_selected_column_mut();
        let cond = if move_next {
            col_idx < cols_len - 1
        } else {
            col_idx > 0
        };
        if cond && !column.tasks.is_empty() {
            let t = column.tasks.remove(column.selected_task_idx);
            column.select_previous_task();
            if move_next {
                self.select_next_column();
            } else {
                self.select_previous_column();
            }
            let col = self.get_selected_column_mut();
            col.tasks.push(t);
            col.select_last_task();
        }
    }

    /// Returns the move task up of this [`State`].
    ///
    /// # Panics
    ///
    /// We have conditions to ensure this doesn't panic but we still unwrap()
    pub fn move_task_up(&mut self) -> bool {
        let column = self.get_selected_column_mut();
        if column.selected_task_idx > 0 {
            column.tasks.swap(column.selected_task_idx, column.selected_task_idx - 1);
            column.selected_task_idx -= 1;
            let task1_id = column.get_selected_task().unwrap().id;
            let task2_id = column.get_next_task().unwrap().id;
            let col_id = column.id;
            let task_idx = column.selected_task_idx;
            self.db_conn.swap_task_order(task1_id, task2_id);
            self.db_conn.set_selected_task_for_column(task_idx, col_id);
            true
        } else {
            false
        }
    }

    /// Returns the move task down of this [`State`].
    ///
    /// # Panics
    ///
    /// We have conditions to ensure this doesn't panic but we still unwrap()
    pub fn move_task_down(&mut self) -> bool {
        let column = self.get_selected_column_mut();
        if column.selected_task_idx < column.tasks.len().saturating_sub(1) {
            let task_idx = column.selected_task_idx;
            column.tasks.swap(task_idx, task_idx + 1);
            column.selected_task_idx += 1;
            let task1_id = column.get_selected_task().unwrap().id;
            let task2_id = column.get_previous_task().unwrap().id;
            let col_id = column.id;
            self.db_conn.swap_task_order(task1_id, task2_id);
            self.db_conn.set_selected_task_for_column(task_idx, col_id);
            true
        } else {
            false
        }
    }

    /// Returns the move task previous column of this [`State`].
    ///
    /// # Panics
    ///
    /// We have conditions to ensure this doesn't panic but we still unwrap()
    pub fn move_task_previous_column(&mut self) {
        let first_col = self.get_selected_column_mut();
        let task_idx = first_col.selected_task_idx.saturating_sub(1);
        let col_id = first_col.id;
        self.db_conn.set_selected_task_for_column(task_idx, col_id);
        self.move_task_to_column(false);
        self.db_conn.move_task_to_column(
            self.get_selected_column().get_selected_task().unwrap(),
            self.get_selected_column(),
        );
        self.db_conn.set_selected_column(self.selected_column_idx);
    }

    /// Returns the move task previous column of this [`State`].
    ///
    /// # Panics
    ///
    /// We have conditions to ensure this doesn't panic but we still unwrap()
    pub fn move_task_next_column(&mut self) {
        let first_col = self.get_selected_column_mut();
        let task_idx = first_col.selected_task_idx.saturating_sub(1);
        let col_id = first_col.id;
        self.db_conn.set_selected_task_for_column(task_idx, col_id);
        self.move_task_to_column(true);
        self.db_conn.move_task_to_column(
            self.get_selected_column().get_selected_task().unwrap(),
            self.get_selected_column(),
        );
        self.db_conn.set_selected_column(self.selected_column_idx);
    }

    /// Returns the delete task of this [`State`].
    ///
    /// # Panics
    ///
    /// We have conditions to ensure this doesn't panic but we still unwrap()
    pub fn delete_task(&mut self) {
        let column = self.get_selected_column_mut();
        let task_id = column.get_selected_task().unwrap().id;
        column.remove_task();
        let task_idx = column.selected_task_idx;
        let col_id = self.get_selected_column().id;
        self.db_conn.delete_task(task_id);
        self.db_conn.set_selected_task_for_column(task_idx, col_id);
    }
}

impl<'a> Column {
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.select_last_task();
    }

    pub fn remove_task(&mut self) {
        self.tasks.remove(self.selected_task_idx);
        self.select_next_task();
    }

    #[must_use]
    pub fn get_selected_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_task_idx)
    }

    #[must_use]
    pub fn get_previous_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_task_idx - 1)
    }

    #[must_use]
    pub fn get_next_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_task_idx + 1)
    }

    pub fn get_selected_task_mut(&mut self) -> Option<&mut Task> {
        self.tasks.get_mut(self.selected_task_idx)
    }

    pub fn select_previous_task(&mut self) {
        let task_idx = &mut self.selected_task_idx;
        *task_idx = task_idx.saturating_sub(1);
    }

    pub fn select_next_task(&mut self) {
        let task_idx = &mut self.selected_task_idx;
        *task_idx = min(*task_idx + 1, self.tasks.len().saturating_sub(1));
    }

    pub fn select_first_task(&mut self) {
        self.selected_task_idx = 0;
    }

    pub fn select_last_task(&mut self) {
        self.selected_task_idx = self.tasks.len().saturating_sub(1);
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
}

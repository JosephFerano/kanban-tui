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

/// Type used mainly for serialization at this time
#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub name: String,
    pub filepath: String,
    pub selected_column_idx: usize,
    pub columns: Vec<Column>,
}

#[derive(Debug, thiserror::Error)]
pub enum KanbanError {
    #[error("There is something wrong with the json schema, it doesn't match Project struct")]
    BadJson,
    #[error("IO - {0}")]
    Io(#[from] std::io::Error),
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
    pub project: Project,
    pub db_conn: Connection,
    pub quit: bool,
    pub columns: Vec<Column>,
    pub task_edit_state: Option<TaskState<'a>>,
}

impl State<'_> {
    #[must_use]
    pub fn new(db_pool: Connection, project: Project) -> Self {
        State {
            quit: false,
            task_edit_state: None,
            db_conn: db_pool,
            project,
            columns: vec![],
        }
    }
}

impl<'a> Column {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Column {
            // TODO: Get the right ID here
            id: 1,
            name: name.to_owned(),
            tasks: vec![],
            selected_task_idx: 0,
        }
    }

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
        self.selected_task_idx = self.tasks.len() - 1;
    }

    pub fn move_task_up(&mut self) -> bool {
        if self.selected_task_idx > 0 {
            self.tasks
                .swap(self.selected_task_idx, self.selected_task_idx - 1);
            self.selected_task_idx -= 1;
            true
        } else {
            false
        }
    }

    pub fn move_task_down(&mut self) -> bool {
        if self.selected_task_idx < self.tasks.len() - 1 {
            self.tasks
                .swap(self.selected_task_idx, self.selected_task_idx + 1);
            self.selected_task_idx += 1;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn get_task_state_from_curr_selected_task(&self) -> Option<TaskState<'a>> {
        self.get_selected_task().map(|t| TaskState {
            title: TextArea::from(t.title.lines()),
            description: TextArea::from(t.description.lines()),
            focus: TaskEditFocus::Title,
            is_edit: true,
        })
    }
}

impl Project {
    pub async fn load(pool: &Connection) -> Result<Self, KanbanError> {
        let columns = db::get_all_columns(&pool).unwrap();

        Ok(Project {
            name: String::from("Kanban Board"),
            filepath: String::from("path"),
            columns,
            selected_column_idx: 0,
        })
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

    pub fn move_task_previous_column(&mut self) {
        self.move_task_to_column(false);
    }

    pub fn move_task_next_column(&mut self) {
        self.move_task_to_column(true);
    }
}

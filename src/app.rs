// use indexmap::IndexMap;
// use int_enum::IntEnum;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::fs::{File};
use std::io::Read;
use tui_textarea::TextArea;

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub selected_task_idx: usize,
    pub tasks: Vec<Task>,
}

// #[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
    pub title: String,
    pub description: String,
}

impl Default for Task {
    fn default() -> Self {
        Task {
            title: String::new(),
            description: String::new(),
        }
    }
}

/// Type used mainly for serialization at this time
#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub name: String,
    pub filepath: String,
    pub selected_column_idx: usize,
    pub columns: Vec<Column>
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
    pub is_edit: bool
}

impl Default for TaskState<'_> {
    fn default() -> Self {
        TaskState {
            title: TextArea::default(),
            description: TextArea::default(),
            focus: TaskEditFocus::Title,
            is_edit: false
        }
    }
}

pub struct AppState<'a> {
    pub project: Project,
    pub quit: bool,
    pub columns: Vec<Column>,
    pub task_edit_state: Option<TaskState<'a>>,
}

impl AppState<'_> {
    pub fn new(project: Project) -> Self {
        AppState {
            quit: false,
            task_edit_state: None,
            project,
            columns: vec![],
        }
    }
}

impl<'a> Column {
    pub fn new(name: &str) -> Self {
        Column {
            name: name.to_owned(),
            tasks: vec![],
            selected_task_idx: 0,
        }
    }

    pub fn add_task(&mut self, title: String, description: String) {
        let task = Task { title, description };
        self.tasks.push(task);
        self.select_next_task();
    }

    pub fn remove_task(&mut self) {
        self.tasks.remove(self.selected_task_idx);
        self.select_next_task();
    }

    pub fn get_selected_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_task_idx)
    }

    pub fn get_selected_task_mut(&mut self) -> Option<&mut Task> {
        self.tasks.get_mut(self.selected_task_idx)
    }

    pub fn select_previous_task(&mut self) {
        let task_idx = &mut self.selected_task_idx;
        *task_idx = task_idx.saturating_sub(1)
    }

    pub fn select_next_task(&mut self) {
        let task_idx = &mut self.selected_task_idx;
        *task_idx = min(*task_idx + 1, self.tasks.len().saturating_sub(1))
    }

    pub fn select_last_task(&mut self) {
        self.selected_task_idx = self.tasks.len() - 1;
    }

    pub fn get_task_state_from_curr_selected_task(&self) -> Option<TaskState<'a>> {
        self.get_selected_task().map(|t| {
            TaskState {
                title: TextArea::from(t.title.lines()),
                description: TextArea::from(t.description.lines()),
                focus: TaskEditFocus::Title,
                is_edit: true
            }
        })
    }
}

impl Project {
    pub fn new(name: &str, filepath: String) -> Self {
        Project {
            name: name.to_owned(),
            filepath,
            columns: vec![
                Column::new("Todo"),
                Column::new("InProgress"),
                Column::new("Done"),
                Column::new("Ideas"),
            ],
            selected_column_idx: 0,
        }
    }

    fn load_from_json(json: &str) -> Result<Self, KanbanError> {
        serde_json::from_str(json).map_err(|_| KanbanError::BadJson)
    }

    pub fn load(path: String, mut file: &File) -> Result<Self, KanbanError> {
        let mut json = String::new();
        file.read_to_string(&mut json)?;
        if json.trim().is_empty() {
            Ok(Project::new("", path))
        } else {
            Self::load_from_json(&json)
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(&self.filepath, json).unwrap();
    }

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
        self.selected_column_idx = min(
            self.selected_column_idx + 1,
            self.columns.len() - 1,
        );
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
        if cond && column.tasks.len() > 0 {
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
            self.save();
        }
    }

    pub fn move_task_previous_column(&mut self) {
        self.move_task_to_column(false)
    }

    pub fn move_task_next_column(&mut self) {
        self.move_task_to_column(true)
    }

    pub fn move_task_up(&mut self) {
        let column = self.get_selected_column_mut();
        if column.selected_task_idx > 0 {
            column.tasks.swap(column.selected_task_idx, column.selected_task_idx - 1);
            column.selected_task_idx = column.selected_task_idx - 1;
            self.save();
        }
    }

    pub fn move_task_down(&mut self) {
        let column = self.get_selected_column_mut();
        if column.selected_task_idx < column.tasks.len() - 1 {
            column.tasks.swap(column.selected_task_idx, column.selected_task_idx + 1);
            column.selected_task_idx = column.selected_task_idx + 1;
            self.save();
        }
    }
}

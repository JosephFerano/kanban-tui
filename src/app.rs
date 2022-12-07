use indexmap::IndexMap;
use int_enum::IntEnum;
use serde::{Deserialize, Serialize};
use std::cmp::min;

#[cfg(test)]
mod tests;

#[repr(usize)]
#[derive(
    Deserialize, Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, IntEnum,
)]
pub enum TaskStatus {
    Todo = 0,
    InProgress = 1,
    Done = 2,
    Ideas = 3,
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
    pub tasks_per_column: IndexMap<TaskStatus, Vec<Task>>,
}

#[derive(Debug, thiserror::Error)]
pub enum KanbanError {
    #[error("There is something wrong with the json schema, it doesn't match Project struct")]
    BadJson,
    #[error("Some form of IO error occured: {0}")]
    Io(#[from] std::io::Error),
}

impl Project {
    pub fn new(name: &str) -> Self {
        Project {
            name: name.to_owned(),
            tasks_per_column: IndexMap::from([
                (TaskStatus::Done, vec![]),
                (TaskStatus::Todo, vec![]),
                (TaskStatus::InProgress, vec![]),
                (TaskStatus::Ideas, vec![]),
            ]),
        }
    }

    fn load_from_json(json: &str) -> Result<Self, KanbanError> {
        serde_json::from_str(json).map_err(|_| KanbanError::BadJson)
    }

    pub fn load() -> Result<Self, KanbanError> {
        let json = std::fs::read_to_string("kanban-tui.json")?;
        Self::load_from_json(&json)
    }

    pub fn add_task(&mut self, status: TaskStatus, task: Task) {
        self.tasks_per_column.entry(status).or_default().push(task);
    }

    pub fn save(&self) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write("kanban-tui.json", json).unwrap();
    }
}

impl Default for Project {
    fn default() -> Self {
        Project {
            name: String::new(),
            tasks_per_column: IndexMap::new(),
        }
    }
}

pub struct AppState {
    pub selected_column: usize,
    pub selected_task: [usize; 4],
    pub project: Project,
    pub quit: bool,
    pub popup_text: Option<String>,
}

impl AppState {
    pub fn new(project: Project) -> Self {
        AppState {
            selected_column: 0,
            selected_task: [0, 0, 0, 0],
            quit: false,
            popup_text: None,
            project,
        }
    }

    fn selected_task_idx(&self) -> usize {
        self.selected_task[self.selected_column]
    }

    fn selected_task_idx_mut(&mut self) -> &mut usize {
        &mut self.selected_task[self.selected_column]
    }

    pub fn get_tasks_in_active_column(&self) -> &[Task] {
        let column: TaskStatus = TaskStatus::from_int(self.selected_column).unwrap().clone();
        self.project.tasks_per_column.get(&column).unwrap()
    }

    pub fn get_tasks_in_active_column_mut(&mut self) -> &mut Vec<Task> {
        let column: TaskStatus = TaskStatus::from_int(self.selected_column).unwrap().clone();
        self.project.tasks_per_column.get_mut(&column).unwrap()
    }

    pub fn get_selected_task(&self) -> Option<&Task> {
        let tasks = self.get_tasks_in_active_column();
        tasks.get(self.selected_task_idx())
    }

    pub fn select_previous_task(&mut self) {
        *self.selected_task_idx_mut() = self.selected_task_idx().saturating_sub(1)
    }

    pub fn select_next_task(&mut self) {
        let tasks = self.get_tasks_in_active_column();
        if tasks.len() > 0 {
            let mins = min(self.selected_task_idx() + 1, tasks.len() - 1);
            *self.selected_task_idx_mut() = mins;
        }
    }

    pub fn select_previous_column(&mut self) {
        self.selected_column = self.selected_column.saturating_sub(1);
    }

    pub fn select_next_column(&mut self) {
        self.selected_column = min(
            self.selected_column + 1,
            self.project.tasks_per_column.len() - 1,
        )
    }

    pub fn move_task_previous_column(&mut self) {
        let tasks = self.get_tasks_in_active_column();
        let task_idx = self.selected_task_idx();
        if self.selected_column > 0 && tasks.len() > 0 && task_idx.clone() < tasks.len() {
            let task = self.get_tasks_in_active_column_mut().remove(task_idx);
            *self.selected_task_idx_mut() = self.selected_task_idx().saturating_sub(1);
            self.select_previous_column();
            let target_tasks = self.get_tasks_in_active_column_mut();
            target_tasks.push(task);
            *self.selected_task_idx_mut() = target_tasks.len() - 1;
            self.project.save();
        }
    }

    pub fn move_task_next_column(&mut self) {
        let tasks = self.get_tasks_in_active_column();
        let task_idx = self.selected_task_idx();
        if self.selected_column < self.project.tasks_per_column.len()
            && tasks.len() > 0
            && task_idx < tasks.len()
        {
            let task = self.get_tasks_in_active_column_mut().remove(task_idx);
            *self.selected_task_idx_mut() = self.selected_task_idx().saturating_sub(1);
            self.select_next_column();
            let target_tasks = self.get_tasks_in_active_column_mut();
            target_tasks.push(task);
            *self.selected_task_idx_mut() = target_tasks.len() - 1;
            self.project.save();
        }
    }

    pub fn move_task_up(&mut self) {
        let task_idx = self.selected_task_idx();
        if task_idx > 0 {
            let tasks = self.get_tasks_in_active_column_mut();
            tasks.swap(task_idx, task_idx - 1);
            *self.selected_task_idx_mut() = task_idx - 1;
            self.project.save();
        }
    }

    pub fn move_task_down(&mut self) {
        let task_idx = self.selected_task_idx();
        let tasks = self.get_tasks_in_active_column_mut();
        if task_idx < tasks.len() - 1 {
            tasks.swap(task_idx, task_idx + 1);
            *self.selected_task_idx_mut() = task_idx + 1;
            self.project.save();
        }
    }
}

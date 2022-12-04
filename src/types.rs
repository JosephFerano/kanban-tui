use std::cmp::min;
use indexmap::IndexMap;
use int_enum::IntEnum;
use serde::{Deserialize, Serialize};

#[repr(usize)]
#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, IntEnum)]
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

impl Project {
    fn new(name: &str) -> Self {
        Project {
            name: name.to_owned(),
            tasks_per_column: IndexMap::from(
                [(TaskStatus::Done, vec![]),
                    (TaskStatus::Todo, vec![]),
                    (TaskStatus::InProgress, vec![]),
                    (TaskStatus::Ideas, vec![])],
            ),
        }
    }

    pub fn load() -> Self {
        let json = std::fs::read_to_string("kanban-tui.json")
            .expect("Could not read json file");
        serde_json::from_str(&json)
            .expect("There is something wrong with the json schema, it doesn't match Project struct")
    }

    pub fn add_task(&mut self, status: TaskStatus, task: Task) {
        self.tasks_per_column.entry(status).or_default().push(task);
    }

    /// Comment out cause this is dangerous
    pub fn save() {
        // let mut project = Project::new("Kanban Tui");
        // project.add_task(Task::default());
        // project.add_task(Task::default());
        // let json = serde_json::to_string_pretty(&project).unwrap();
        // std::fs::write("./project.json", json).unwrap();
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
}

impl AppState {
    pub fn new(project: Project) -> Self {
        AppState {
            selected_column: 0,
            selected_task: [0, 0, 0, 0],
            quit: false,
            project: project,
        }
    }

    pub fn get_tasks_in_active_column(&self) -> &Vec<Task> {
        let column: TaskStatus = TaskStatus::from_int(self.selected_column).unwrap().clone();
        self.project.tasks_per_column.get(&column).unwrap()
    }

    pub fn select_previous_task(&mut self) {
        self.selected_task[self.selected_column] = self.selected_task[self.selected_column].saturating_sub(1)
    }

    pub fn select_next_task(&mut self) {
        let tasks = self.get_tasks_in_active_column();
        if tasks.len() > 0 {
            let mins = min(self.selected_task[self.selected_column] + 1, tasks.len() - 1);
            self.selected_task[self.selected_column] = mins;
        }
    }

    pub fn select_previous_column(&mut self) {
        self.selected_column = self.selected_column.saturating_sub(1);
    }

    pub fn select_next_column(&mut self) {
        self.selected_column = min(self.selected_column + 1, self.project.tasks_per_column.len() - 1)
    }
}


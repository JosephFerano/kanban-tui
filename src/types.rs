use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum TaskStatus {
    Done,
    Todo,
    InProgress,
    Testing,
    Backlog,
}

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

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub name: String,
    pub tasks: IndexMap<TaskStatus, Vec<Task>>,
}

impl Project {
    fn new(name: &str) -> Self {
        Project {
            name: name.to_owned(),
            tasks: IndexMap::from(
                [(TaskStatus::Done, vec![]),
                    (TaskStatus::Todo, vec![]),
                    (TaskStatus::InProgress, vec![]),
                    (TaskStatus::Testing, vec![]),
                    (TaskStatus::Backlog, vec![])],
            ),
        }
    }

    fn add_task(&mut self, status: TaskStatus, task: Task) {
        self.tasks.entry(status).or_default().push(task);
    }
}

impl Default for Project {
    fn default() -> Self {
        Project {
            name: String::new(),
            tasks: IndexMap::new(),
        }
    }
}

impl Project {
    pub fn load() -> Self {
        let json = std::fs::read_to_string("kanban-tui.json")
            .expect("Could not read json file");
        serde_json::from_str(&json)
            .expect("There is something wrong with the json schema, it doesn't match Project struct")
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

pub struct AppState {
    pub selected_column: usize,
    pub selected_task: [u8; 5],
    pub current_project: Project,
    pub quit: bool,
}

impl AppState {
    pub fn new(project: Project) -> Self {
        AppState {
            selected_column: 0,
            selected_task: [0, 0, 0, 0, 0],
            quit: false,
            current_project: project,
        }
    }
}


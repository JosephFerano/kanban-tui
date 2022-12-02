use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
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
    pub status: TaskStatus,
}

impl Default for Task {
    fn default() -> Self {
        Task {
            title: String::new(),
            description: String::new(),
            status: TaskStatus::Backlog,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub name: String,
    pub tasks: Vec<Task>,
}

impl Project {
    fn new(name: &str) -> Self {
        Project { name: name.to_owned() , tasks: Vec::new() }
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }
}

impl Default for Project {
    fn default() -> Self {
        Project {
            name: String::new(),
            tasks: Vec::new(),
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
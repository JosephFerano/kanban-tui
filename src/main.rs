#![deny(rust_2018_idioms)]
#![allow(unused_imports)]
#![allow(dead_code)]
use kanban_tui::*;
use crossterm::{
    event::*,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen
    },
};
use std::{
    io::{self, Write},
    env,
    path::PathBuf,
    fs::{File, OpenOptions},
    error::Error
};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use clap::{Parser, ValueHint::FilePath};

const DEFAULT_DATABASE_NAME: &str = "kanban.json";

#[derive(Debug, Parser)]
#[command(name = "kanban")]
/// kanban-tui is a simple, interactive TUI based task manager using kanban columns
pub struct CliArgs {
    #[arg(short('d'), long("database"), value_name="DATABASE", value_hint=FilePath)]
    /// Path to the
    pub filepath: Option<PathBuf>
}

fn prompt_project_init(default_name: &str) -> (String, io::Result<File>) {
    let mut input = String::new();

    println!("Database not found, select the name of the database if it exists or enter a name to start a new project");
    print!("Database name (default: {}): ", default_name);
    io::stdout().flush().unwrap();

    let result = io::stdin().read_line(&mut input);
    let input = input.trim();

    let filename =
        match result {
            Ok(b) if b > 0 && !input.is_empty() => input,
            _ => default_name
        };

    // TODO: This might be a good time to prompt the user if they want
    // to change the default column names

    (filename.to_string(),
     OpenOptions::new()
     .write(true)
     .read(true)
     .create(true)
     .open(filename))
}

fn main() -> anyhow::Result<(), Box<dyn Error>> {
    let (filepath, file) =
        match CliArgs::parse() {
            CliArgs { filepath: Some(filepath) } => {
                let fpath = filepath.into_os_string().into_string().unwrap();
                let file = OpenOptions::new()
                    .write(true)
                    .read(true)
                    .open(&fpath);

                match file {
                    Ok(f) => (fpath, f),
                    Err(_) => {
                        let (fp, fname) = prompt_project_init(&fpath);
                        (fp, fname.unwrap())
                    }
                }
            },
            CliArgs { filepath: None } => {
                let file = OpenOptions::new()
                    .write(true)
                    .read(true)
                    .open(DEFAULT_DATABASE_NAME);
                if let Ok(f) = file {
                    (DEFAULT_DATABASE_NAME.to_string(), f)
                } else {
                    let (fp, fname) = prompt_project_init(DEFAULT_DATABASE_NAME);
                    (fp, fname.unwrap())
                }
            }
        };
    let mut state = AppState::new(Project::load(filepath, &file)?);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    while !state.quit {
        terminal.draw(|f| kanban_tui::draw(f, &mut state))?;
        kanban_tui::handle_input(&mut state)?;
    }

    // state.project.save();

    // restore terminal
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

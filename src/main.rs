#![deny(rust_2018_idioms)]
use clap::{Parser, ValueHint::FilePath};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use kanban_tui::{Project, State};
use sqlx::sqlite::SqlitePool;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};
use tui::backend::CrosstermBackend;
use tui::Terminal;

const DEFAULT_DATABASE_NAME: &str = "kanban.json";

#[derive(Debug, Parser)]
#[command(name = "kanban")]
/// kanban-tui is a simple, interactive TUI based task manager using kanban columns
pub struct CliArgs {
    #[arg(value_name="DATABASE", value_hint=FilePath, index=1)]
    /// Path to the
    pub filepath: Option<PathBuf>,
}

// TODO: This should just return a struct beacuse we should add a
// "should_quit" thing instead of calling exit(0) here
fn prompt_project_init(default_name: &str) -> (String, io::Result<File>) {
    let mut input = String::new();

    println!("Database not found, select the name of the database if it exists or enter a name to start a new project");
    print!("Database name (default: {default_name}): ");
    io::stdout().flush().unwrap();

    let result = io::stdin().read_line(&mut input);
    let input = input.trim();

    let filename = match result {
        Ok(b) if b == 0 => std::process::exit(0),
        Ok(b) if b > 0 && !input.is_empty() => input,
        _ => default_name,
    };

    // TODO: This might be a good time to prompt the user if they want
    // to change the default column names

    (
        filename.to_string(),
        OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(filename),
    )
}

#[async_std::main]
async fn main() -> anyhow::Result<(), Box<dyn Error>> {
    let (_filepath, _file) = match CliArgs::parse() {
        CliArgs {
            filepath: Some(filepath),
        } => {
            let fpath = filepath.into_os_string().into_string().unwrap();
            let file = OpenOptions::new().write(true).read(true).open(&fpath);

            if let Ok(f) = file {
                (fpath, f)
            } else {
                let (fp, fname) = prompt_project_init(&fpath);
                (fp, fname.unwrap())
            }
        }
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

    let db_pool = SqlitePool::connect("sqlite:db.sqlite").await?;

    let project = Project::load2(&db_pool).await?;
    let mut state = State::new(db_pool, project);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    while !state.quit {
        terminal.draw(|f| kanban_tui::draw(f, &mut state))?;
        kanban_tui::handle(&mut state)?;
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

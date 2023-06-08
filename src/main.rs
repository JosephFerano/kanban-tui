#![deny(rust_2018_idioms)]
#![allow(unused_imports)]
#![allow(dead_code)]
use kanban_tui::*;
use crossterm::{
    event::*,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, env, path::PathBuf, fs::{File, OpenOptions}, error::Error};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "kanban")]
/// kanban-tui is a simple, interactive TUI based task manager using kanban columns
pub struct CliArgs {
    #[arg(short('d'), long("database"), value_name="DATABASE")]
    /// Path to the
    pub filepath: Option<PathBuf>
}

fn prompt_project_init() -> (String, io::Result<File>) {
    let _msg = "Database not found, would you like to start a new project?";
    let filepath = "kanban.json";
    (filepath.to_string(),
     OpenOptions::new()
     .write(true)
     .read(true)
     .create(true)
     .open(filepath))
}

fn main() -> anyhow::Result<(), Box<dyn Error>> {
    let (filepath, file) =
        match CliArgs::parse() {
            CliArgs { filepath: Some(filepath) } => {
                let fpath = filepath.into_os_string().into_string().unwrap();
                let file = OpenOptions::new()
                    .write(true)
                    .read(true)
                    .create(true)
                    .open(&fpath);

                match file {
                    Ok(f) => (fpath, f),
                    Err(_) => {
                        let (fp, fname) = prompt_project_init();
                        (fp, fname.unwrap())
                    }
                }
            },
            CliArgs { filepath: None } => {
                let (fp, fname) = prompt_project_init();
                (fp, fname.unwrap())
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

    state.project.save();

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

#![deny(rust_2018_idioms)]
use clap::{Parser, ValueHint::FilePath};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use kanban_tui::{Project, State};
use rusqlite::Connection;
use std::{error::Error, io, path::PathBuf};
use tui::backend::CrosstermBackend;
use tui::Terminal;

#[derive(Debug, Parser)]
#[command(name = "kanban")]
/// kanban-tui is a simple, interactive TUI based task manager using kanban columns
pub struct CliArgs {
    #[arg(value_name="DATABASE", value_hint=FilePath, index=1)]
    /// Path to the SQLite database
    pub filepath: Option<PathBuf>,
}

// TODO: We either make everything async or we remove the dependency
#[async_std::main]
async fn main() -> anyhow::Result<(), Box<dyn Error>> {
    let dbpath = CliArgs::parse()
        .filepath
        .unwrap_or(PathBuf::from("./kanban.db"));

    let migrate = !dbpath.exists();

    let mut conn = Connection::open(dbpath)?;

    if migrate {
        let migrations = include_str!("../sql/migrations.sql");
        let migrations: Vec<&str> = migrations.split(";").collect();
        let tx = conn.transaction()?;
        for m in migrations {
            if !m.trim().is_empty() {
                tx.execute(m, ())?;
            }
        }
        tx.commit()?;
    }

    let project = Project::load(&conn).await?;
    let mut state = State::new(conn, project);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    while !state.quit {
        terminal.draw(|f| kanban_tui::draw(f, &mut state))?;
        kanban_tui::handle(&mut state)?;
    }

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

//! Manage your TODOs with kanban columns right from the terminal.
//!
//! kanban-tui is a TUI based application using [`ratatui`] for
//! rendering the UI and capturing user input interaction, with
//! [`Crossterm`] as the lower-level backend system for terminal
//! text-based interfaces. The data is saved to a SQLite database
//! ideally placed in the root of your project. For this the
//! [`rusqlite`] crate provides the bindings to handle all the data
//! persistence.
//!
//! [`ratatui`]: https://crates.io/crates/ratatui
//! [`Crossterm`]: https://crates.io/crates/crossterm
//! [`rusqlite`]: https://crates.io/crates/rusqlite

#![deny(rust_2018_idioms)]
mod app;
mod db;
mod input;
mod ui;

pub use app::*;
pub use db::*;
pub use input::handle_user_keypress;
pub use ui::draw_ui_from_state;

use anyhow::Error;
use int_enum::IntEnum;
use rusqlite::Connection;
use std::cmp::min;
use tui_textarea::TextArea;

use crate::db;

/// Represents a kanban column containing the tasks and other metadata.
#[derive(Debug)]
pub struct Column {
    /// Id provided by the database
    pub id: i64,
    /// The name used for the title in the UI
    pub name: String,
    /// The currently selected [`Task`], which keeps track of the
    /// user's position in a column when the go from one to another
    pub selected_task_idx: usize,
    /// The collection of [`Task`]
    pub tasks: Vec<Task>,
}

/// Basic TODO task with a title and a description.
#[derive(Clone, Default, Debug)]
pub struct Task {
    /// Id provided by the database
    pub id: i64,
    /// Title of the [`Task`]
    pub title: String,
    /// Description of the [`Task`]
    pub description: String,
}

/// The number of TaskEditFocus variants, used so we can "wrap around"
/// with modulo when cycling through tasks with Tab/Backtab.
pub const EDIT_WINDOW_FOCUS_STATES: i8 = 4;

/// Used to track the focus of the form field in the task edit window.
#[repr(i8)]
#[derive(Debug, IntEnum, Copy, Clone)]
pub enum TaskEditFocus {
    /// Title text input line
    Title = 0,
    /// Description text input box
    Description = 1,
    /// Confirm changes button
    ConfirmBtn = 2,
    /// Cancel changes button
    CancelBtn = 3,
}

/// Represents the transient state of a task while it is being editing
/// by the user through the UI.
pub struct TaskState<'a> {
    /// The title of the Task
    pub title: TextArea<'a>,
    /// The description of the Task
    pub description: TextArea<'a>,
    /// Where the current focus of the task edit form is
    pub focus: TaskEditFocus,
    /// Used to decide if the user is editing an existing task or
    /// creating a new one
    pub is_edit: bool,
}

impl Default for TaskState<'_> {
    fn default() -> Self {
        TaskState {
            title: TextArea::default(),
            description: TextArea::default(),
            focus: TaskEditFocus::Title,
            is_edit: false,
        }
    }
}

/// Holds the application's state, including all columns and the
/// database connection.
pub struct State<'a> {
    /// The name of the project, currently derived from the name of
    /// the current working directory
    pub project_name: String,
    /// The index of the currently selected [`Column`]
    pub selected_column_idx: usize,
    /// A vec of all the [`Column`]s
    pub columns: Vec<Column>,
    /// The [`db::DBConn`] wrapping a [`rusqlite::Connection`]
    pub db_conn: db::DBConn,
    /// Flag to check on each loop whether we should exit the app
    pub quit: bool,
    /// If [`Some(TaskState)`] then we are in the task edit form window
    pub task_edit_state: Option<TaskState<'a>>,
}

impl<'a> State<'a> {
    /// Creates a new [`State`].
    ///
    /// # Errors
    ///
    /// Returns an error if we can't read the database columns
    pub fn new(conn: Connection) -> Result<Self, Error> {
        let db_conn = db::DBConn::new(conn);
        let columns = db_conn.get_all_columns()?;
        let selected_column = db_conn.get_selected_column()?;

        let project_name = std::env::current_dir()?
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("KANBAN PROJECT")
            .to_string();

        Ok(State {
            project_name,
            columns,
            selected_column_idx: selected_column,
            quit: false,
            task_edit_state: None,
            db_conn,
        })
    }

    /// Returns a reference to the currently selected [`Column`].
    #[must_use]
    pub fn get_selected_column(&self) -> &Column {
        &self.columns[self.selected_column_idx]
    }

    /// Returns a mutable reference to the currently selected
    /// [`Column`].
    pub fn get_selected_column_mut(&mut self) -> &mut Column {
        &mut self.columns[self.selected_column_idx]
    }

    /// Selects the [`Column`] on the left. Does nothing if on the
    /// first column.
    pub fn select_column_left(&mut self) -> Result<(), Error> {
        self.selected_column_idx = self.selected_column_idx.saturating_sub(1);
        self.db_conn.set_selected_column(self.selected_column_idx)
    }

    /// Selects the [`Column`] on the right. Does nothing if on the
    /// last column.
    pub fn select_column_right(&mut self) -> Result<(), Error> {
        self.selected_column_idx = min(self.selected_column_idx + 1, self.columns.len() - 1);
        self.db_conn.set_selected_column(self.selected_column_idx)
    }

    /// Returns a reference to the currently selected [`Task`].
    /// Returns `None` if the current [`Column::tasks`] is empty.
    #[must_use]
    pub fn get_selected_task(&self) -> Option<&Task> {
        let column = self.get_selected_column();
        column.tasks.get(column.selected_task_idx)
    }

    /// Returns a reference to the [`Task`] above the current one.
    /// Returns `None` if it's the first task on the list
    #[must_use]
    pub fn get_task_above(&self) -> Option<&Task> {
        let column = self.get_selected_column();
        if column.selected_task_idx > 0 {
            column.tasks.get(column.selected_task_idx - 1)
        } else {
            None
        }
    }

    /// Returns a reference to the [`Task`] below the current one.
    /// Returns `None` if it's the last task on the list
    #[must_use]
    pub fn get_task_below(&self) -> Option<&Task> {
        let column = self.get_selected_column();
        column.tasks.get(column.selected_task_idx + 1)
    }

    /// Returns a mutable reference to the currently selected
    /// [`Task`]. Returns `None` if the current [`Column::tasks`] is
    /// empty.
    pub fn get_selected_task_mut(&mut self) -> Option<&mut Task> {
        let column = self.get_selected_column_mut();
        column.tasks.get_mut(column.selected_task_idx)
    }

    /// Selects the [`Task`] above the current one. Does nothing if
    /// it's the first task on the list
    pub fn select_task_above(&mut self) -> Result<(), Error> {
        let column = self.get_selected_column_mut();
        column.selected_task_idx = column.selected_task_idx.saturating_sub(1);

        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        self.db_conn
            .set_selected_task_for_column(task_idx, col_id)?;
        Ok(())
    }

    /// Selects the [`Task`] below the current one. Does nothing if
    /// it's the last task on the list
    pub fn select_task_below(&mut self) -> Result<(), Error> {
        let column = self.get_selected_column_mut();
        column.selected_task_idx = min(
            column.selected_task_idx + 1,
            column.tasks.len().saturating_sub(1),
        );

        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        self.db_conn
            .set_selected_task_for_column(task_idx, col_id)?;
        Ok(())
    }

    /// Selects the [`Task`] at the beginning of the list, no matter
    /// where you are in the current [`Column`].
    pub fn select_first_task(&mut self) -> Result<(), Error> {
        let column = self.get_selected_column_mut();
        column.selected_task_idx = 0;

        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        self.db_conn
            .set_selected_task_for_column(task_idx, col_id)?;
        Ok(())
    }

    /// Selects the [`Task`] at the end of the list, no matter
    /// where you are in the current [`Column`].
    pub fn select_last_task(&mut self) -> Result<(), Error> {
        let column = self.get_selected_column_mut();
        column.selected_task_idx = column.tasks.len().saturating_sub(1);

        let task_idx = column.selected_task_idx;
        let col_id = column.id;
        self.db_conn
            .set_selected_task_for_column(task_idx, col_id)?;
        Ok(())
    }

    /// Helper method to construct a [`TaskState`]. Used when we are
    /// going to edit an existing [`Task`]. Returns `None` if the
    /// [`Column`] is empty.
    #[must_use]
    pub fn get_task_state_from_current(&self) -> Option<TaskState<'a>> {
        self.get_selected_task().map(|t| TaskState {
            title: TextArea::from(t.title.lines()),
            description: TextArea::from(t.description.lines()),
            focus: TaskEditFocus::Title,
            is_edit: true,
        })
    }

    /// Moves the current [`Task`] up the list towards the top. Does
    /// nothing if it's the first task.
    pub fn move_task_up(&mut self) -> Result<(), Error> {
        self.move_task(false)
    }

    /// Moves the current [`Task`] down the list towards the bottom. Does
    /// nothing if it's the last task.
    pub fn move_task_down(&mut self) -> Result<(), Error> {
        self.move_task(true)
    }

    /// Private function to handle saving the current [`Task`]'s
    /// state.
    fn move_task(&mut self, is_down: bool) -> Result<(), Error> {
        let other_task = if is_down {
            self.get_task_below()
        } else {
            self.get_task_above()
        };
        if let (Some(task1), Some(task2)) = (self.get_selected_task(), other_task) {
            let t1_id = task1.id;
            let t2_id = task2.id;
            let column = self.get_selected_column_mut();
            let task_idx = column.selected_task_idx;

            let other_idx = if is_down { task_idx + 1 } else { task_idx - 1 };
            column.tasks.swap(task_idx, other_idx);
            if is_down {
                column.selected_task_idx += 1;
            } else {
                column.selected_task_idx -= 1;
            }
            let task_idx = column.selected_task_idx;

            let col_id = column.id;
            self.db_conn.swap_task_order(t1_id, t2_id)?;
            self.db_conn
                .set_selected_task_for_column(task_idx, col_id)?;
       }
        Ok(())
    }

    /// Moves the current [`Task`] to the [`Column`] on the left. Does
    /// nothing if it's the first column.
    pub fn move_task_column_left(&mut self) -> Result<(), Error> {
        self.move_task_to_column(false)
    }

    /// Moves the current [`Task`] to the [`Column`] on the right. Does
    /// nothing if it's the last column.
    pub fn move_task_column_right(&mut self) -> Result<(), Error> {
        self.move_task_to_column(true)
    }

    fn move_task_to_column(&mut self, move_right: bool) -> Result<(), Error> {
        let can_move_right = move_right && self.selected_column_idx < self.columns.len() - 1;
        let can_move_left = !move_right && self.selected_column_idx > 0;

        let first_col = self.get_selected_column_mut();
        if first_col.tasks.is_empty() || !can_move_right && !can_move_left {
            // We're at the bounds so just ignore
            return Ok(());
        }
        let t = first_col.tasks.remove(first_col.selected_task_idx);

        // Only move it if it was the last task
        if first_col.selected_task_idx == first_col.tasks.len() {
            self.select_task_above()?;
        }

        if move_right {
            self.select_column_right()?;
        } else {
            self.select_column_left()?;
        }

        let col = self.get_selected_column_mut();
        col.tasks.push(t);
        self.select_last_task()?;
        if let Some(task) = self.get_selected_task() {
            let col = self.get_selected_column();
            self.db_conn.move_task_to_column(task, col)?;
            self.db_conn.set_selected_column(self.selected_column_idx)?;
        }
        Ok(())
    }

    /// Inserts a new [`Task`] into [`Column::tasks`] at the bottom of
    /// the list and saves the state to the DB.
    pub fn add_new_task(&mut self, title: String, description: String) -> Result<(), Error> {
        let col_id = self.get_selected_column().id;
        let task = self.db_conn.create_new_task(title, description, col_id)?;

        self.select_last_task()?;
        let selected_task_idx = self.get_selected_column().selected_task_idx;
        self.db_conn
            .set_selected_task_for_column(selected_task_idx, col_id)?;

        self.get_selected_column_mut().tasks.push(task);
        self.select_last_task()?;
        Ok(())
    }

    /// Edits the selected [`Task`] changing only it's title and/or
    /// description. Does nothing if the [`Column`] is empty.
    pub fn edit_task(&mut self, title: String, description: String) -> Result<(), Error> {
        if let Some(selected_task) = self.get_selected_task_mut() {
            selected_task.title = title;
            selected_task.description = description;
        }
        if let Some(task) = self.get_selected_task() {
            self.db_conn.update_task_text(task)?;
        }
        Ok(())
    }

    /// Deletes the selected [`Task`] from the list. Does nothing if
    /// the [`Column`] is empty.
    pub fn delete_task(&mut self) -> Result<(), Error> {
        if let Some(task) = self.get_selected_task() {
            let task_id = task.id;
            let column = self.get_selected_column_mut();
            let mut task_idx = column.selected_task_idx;
            let col_id = column.id;

            column.tasks.remove(task_idx);

            if column.selected_task_idx >= column.tasks.len() {
                self.select_task_above()?;
                task_idx = task_idx.saturating_sub(1);
            }

            self.db_conn.delete_task(task_id)?;
            self.db_conn
                .set_selected_task_for_column(task_idx, col_id)?;
        }
        Ok(())
    }
}

use crate::{Column, Task};
use anyhow::Error;
use rusqlite::{params, Connection, Result};
use std::ops::{Deref, DerefMut};

/// Simple one field struct to wrap a  [`rusqlite::Connection`] so we
/// can assign our own methods.
pub struct DBConn(
    /// Unfortunately remains public for now so we can do integration tests
    /// and reuse to simulate exiting and relaunching the app.
    pub Connection
);

impl DerefMut for DBConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for DBConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DBConn {
    pub fn new(conn: Connection) -> Self {
        DBConn(conn)
    }

    /// Query tasks in a [`Column`] by using the column's [`Column::id`].
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn get_tasks_by_column(&self, column_id: i64) -> Result<Vec<Task>> {
        let mut stmt = self.prepare(
            r#"
            select task.id, title, description from task
            where column_id = ?1
            order by sort_order
        "#,
        )?;
        let mut tasks = Vec::new();
        let rows = stmt.query_map([column_id], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
            })
        })?;
        for row in rows {
            tasks.push(row?);
        }
        Ok(tasks)
    }

    /// Uses [`get_tasks_by_column`][`DBConn::get_tasks_by_column`] over
    /// a loop to get all [`Column`] populated with the vec of.
    /// [`Task`]
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn get_all_columns(&self) -> Result<Vec<Column>> {
        let mut stmt = self.prepare("select id, name, selected_task from kb_column")?;
        let columns = stmt
            .query_map((), |row| {
                let id = row.get(0)?;
                Ok(Column {
                    id,
                    tasks: self.get_tasks_by_column(id)?,
                    name: row.get(1)?,
                    selected_task_idx: row.get(2)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(columns)
    }

    /// Insert a new task into the DB given a title and description,
    /// then return the [`Task`] with the ID provided by the DB.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn create_new_task(
        &self,
        title: String,
        description: String,
        column_id: i64,
    ) -> Result<Task> {
        let mut stmt =
            self.prepare(
                "insert into task(title, description, column_id, sort_order)
                values (?1, ?2, ?3,
                  (coalesce(1 +
                    (select sort_order from task
                     where column_id = ?3 order by sort_order desc limit 1),
                  0)))")?;
        stmt.execute(params![title, description, column_id])?;
        let id = self.last_insert_rowid();
        Ok(Task {
            id,
            title,
            description,
        })
    }

    /// Deletes a [`Task`] given it's ID.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn delete_task(&self, task_id: i64) -> Result<()> {
        let mut stmt = self.prepare("delete from task where id = ?1")?;
        stmt.execute([task_id])?;
        Ok(())
    }

    /// Updates an existing [`Task`]'s `title` and `description`.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn update_task_text(&self, task: &Task) -> Result<()> {
        let mut stmt =
            self.prepare("update task set title = ?2, description = ?3 where id = ?1")?;
        stmt.execute((&task.id, &task.title, &task.description))?;
        Ok(())
    }

    /// Moves a [`Task`] to the target [`Column`] and updates the sorting order.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn move_task_to_column(&self, task: &Task, target_column: &Column) -> Result<()> {
        let mut stmt = self
            .prepare(
                "update task
             set
               column_id = ?2,
               sort_order = coalesce(1 +
                 (select sort_order from task
                  where column_id = ?2 order by sort_order desc limit 1),
                  0)
             where task.id = ?1",
            )?;
        stmt.execute((&task.id, &target_column.id))?;
        self.set_selected_task_for_column(target_column.selected_task_idx, target_column.id)?;
        Ok(())
    }

    /// This is a helper function in case we need to debug `sort_order`, because I ran into
    /// a bug when I forgot to insert the `sort_order` when creating a task.
    #[allow(dead_code)]
    fn get_sort_order(&self) -> Result<Vec<(i32, String, usize)>> {
        let mut stmt = self.prepare(
            "select id,title,sort_order from task where column_id = 1")?;
        let mut rows = stmt.query(())?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next()? {
            tasks.push((row.get(0)?, row.get(1)?, row.get(2)?,));
        }
        Ok(tasks)
    }

    /// The order of a [`Task`] in a [`Column`] needs to be saved to
    /// the DB because `SQLite` doesn't have a way to handle the
    /// ordering the internal [`Vec<Task>`] has. This takes the
    /// current sorting order of two tasks and swaps them.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn swap_task_order(&mut self, task1_id: i64, task2_id: i64) -> Result<()> {
        let tx = self.transaction()?;

        tx.execute(
            "create temp table temp_order as select sort_order from task where id = ?1",
            [&task1_id],
        )?;

        tx.execute(
            "update task
             set sort_order = (select sort_order from task where id = ?2)
             where id = ?1",
            (task1_id, task2_id),
        )?;

        tx.execute(
            "update task set sort_order = (select sort_order from temp_order) where id = ?1",
            [&task2_id],
        )?;

        tx.execute("drop table temp_order", ())?;

        tx.commit()?;

        Ok(())
    }

    /// Saves the currently selected column's index to `app_state` so
    /// when the user reloads the project, they start on the
    /// [`Column`] they were last on.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn set_selected_column(&self, column_id: usize) -> Result<(), Error> {
        let mut stmt =
            self.prepare("insert or replace into app_state(key, value) values (?1, ?2)")?;
        stmt.execute((&"selected_column", column_id.to_string()))?;
        Ok(())
    }

    /// Get's the user's last selected [`Column`] before exiting.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn get_selected_column(&self) -> Result<usize> {
        let mut stmt = self.prepare("select value from app_state where key = ?1")?;
        stmt.query_row(["selected_column"], |row| {
            let value: String = row.get::<usize, String>(0)?;
            value.parse::<usize>()
                .map_err(|_| rusqlite::Error::InvalidQuery)
        })
    }

    /// Saves the index currently selected [`Task`] in a [`Column`] so
    /// when the user reloads the project, each column selects the has
    /// the last selected task before switching to another column or
    /// exiting the app.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn set_selected_task_for_column(&self, task_idx: usize, column_id: i64) -> Result<()> {
        let mut stmt = self.prepare("update kb_column set selected_task = ?2 where id = ?1")?;
        stmt.execute((column_id, task_idx))?;
        Ok(())
    }

    /// Get's each [`Column`]'s 's last selected [`Task`] before
    /// switching or exiting.
    ///
    /// # Errors
    ///
    /// Returns an error if something is wrong with the SQL.
    pub fn get_selected_task_for_column(&self, column_id: i32) -> Result<usize> {
        let mut stmt = self.prepare("select selected_task from kb_column where key = ?1")?;
        stmt.query_row([column_id], |row| row.get(0))
    }
}

use crate::{Column, Task};
use anyhow::Error;
use rusqlite::{params, Connection, Result};
use std::ops::{Deref, DerefMut};

pub struct DBConn(Connection);

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

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if something is wrong with the SQL
    pub fn get_tasks_by_column(&self, column_name: &String) -> Result<Vec<Task>> {
        let mut stmt = self.prepare(
            r#"
            select task.id, title, description from task
            join kb_column on column_id = kb_column.id
            where kb_column.name = ?1
            order by sort_order
        "#,
        )?;
        let mut tasks = Vec::new();
        let rows = stmt.query_map([column_name], |row| {
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

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if there are issues with the SQL
    pub fn get_all_columns(&self) -> Result<Vec<Column>> {
        let mut stmt = self.prepare("select id, name, selected_task from kb_column")?;
        let columns = stmt
            .query_map((), |row| {
                let name = row.get(1)?;
                Ok(Column {
                    id: row.get(0)?,
                    tasks: self.get_tasks_by_column(&name)?,
                    name,
                    selected_task_idx: row.get(2)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(columns)
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
    pub fn create_new_task(
        &self,
        title: String,
        description: String,
        column_id: i64,
    ) -> Result<Task> {
        let mut stmt =
            self.prepare("insert into task(title, description, column_id) values (?1, ?2, ?3)")?;
        stmt.execute(params![title, description, column_id])?;
        let id = self.last_insert_rowid();
        Ok(Task {
            id,
            title,
            description,
        })
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
    pub fn delete_task(&self, task_id: i64) -> Result<()> {
        let mut stmt = self.prepare("delete from task where id = ?1")?;
        stmt.execute([task_id])?;
        Ok(())
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
    pub fn update_task_text(&self, task: &Task) -> Result<()> {
        let mut stmt =
            self.prepare("update task set title = ?2, description = ?3 where id = ?1")?;
        stmt.execute((&task.id, &task.title, &task.description))?;
        Ok(())
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
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
            )
            .unwrap();
        stmt.execute((&task.id, &target_column.id)).unwrap();
        self.set_selected_task_for_column(target_column.selected_task_idx, target_column.id)?;
        Ok(())
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
    pub fn swap_task_order(&mut self, task1_id: i64, task2_id: i64) -> Result<()> {
        let tx = self.transaction().unwrap();

        tx.execute(
            "create temp table temp_order as select sort_order from task where id = ?1",
            [&task1_id],
        )
        .unwrap();

        tx.execute(
            "update task set sort_order = (select sort_order from task where id = ?2)
         where id = ?1",
            (task1_id, task2_id),
        )
        .unwrap();

        tx.execute(
            "update task set sort_order = (select sort_order from temp_order) where id = ?1",
            [&task2_id],
        )
        .unwrap();
        tx.execute("drop table temp_order", ()).unwrap();

        tx.commit().unwrap();
        Ok(())
    }

    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
    pub fn set_selected_column(&self, column_id: usize) -> Result<(), Error> {
        let mut stmt =
            self.prepare("insert or replace into app_state(key, value) values (?1, ?2)")?;
        stmt.execute((&"selected_column", column_id.to_string()))?;
        Ok(())
    }

    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
    pub fn get_selected_column(&self) -> Result<usize> {
        let mut stmt = self.prepare("select value from app_state where key = ?1")?;
        stmt.query_row(["selected_column"], |row| {
            let value: String = row.get::<usize, String>(0).unwrap();
            Ok(value.parse::<usize>().unwrap())
        })
    }

    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
    pub fn set_selected_task_for_column(&self, task_idx: usize, column_id: i64) -> Result<()> {
        let mut stmt = self.prepare("update kb_column set selected_task = ?2 where id = ?1")?;
        stmt.execute((column_id, task_idx))?;
        Ok(())
    }

    /// # Panics
    ///
    /// Panics if something goes wrong with the SQL
    pub fn get_selected_task_for_column(&self, column_id: i32) -> Result<usize> {
        let mut stmt = self.prepare("select selected_task from kb_column where key = ?1")?;
        stmt.query_row([column_id], |row| row.get(0))
    }
}

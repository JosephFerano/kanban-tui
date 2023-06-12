use crate::{Task};
use rusqlite::{Connection, Result};

pub fn get_tasks_by_column(conn: &Connection, column_name: &String) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        r#"
            select title, description from task
            join kb_column on column_id = kb_column.id
            where kb_column.name = ?1
        "#,
    )?;
    let mut tasks = Vec::new();
    let rows = stmt.query_map(&[column_name], |row| {
        Ok(Task {
            title: row.get(0)?,
            description: row.get(1)?,
        })
    })?;
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

pub fn get_all_tasks(conn: &Connection) -> Result<Vec<(String, Vec<Task>)>> {
    let mut stmt = conn.prepare("select name from kb_column")?;
    let columns = stmt.query_map((), |row| {
        Ok(row.get::<usize, String>(0)?)
    })?;
    let mut tasks_by_column: Vec<(String, Vec<Task>)> = Vec::new();
    for col in columns {
        let name = &col?;
        let tasks = get_tasks_by_column(conn, name).unwrap();
        tasks_by_column.push((name.to_string(), tasks));
    }
    Ok(tasks_by_column)
}

// pub async fn insert_new_task(pool: &SqlitePool, task: &Task, column: &Column) {
//     // TODO: You have to add the id to the column
//     sqlx::query!("insert into task(title, description, column_id) values (?1, ?2, ?3)", task.title, task.description, 1)
//         .execute(pool)
//         .await
//         .unwrap();
// }

// pub async fn update_task(pool: &SqlitePool, task: &Task) {
//     sqlx::query!("update task set title = ?1, description = ?2", task.title, task.description)
//         .execute(pool)
//         .await
//         .unwrap();
// }

// pub async fn move_task_to_column(pool: &SqlitePool, task: &Task, target_column: &Column) {
//     // TODO: You have to add the id to the column
//     sqlx::query!("update task set column_id = ?1", 1)
//         .execute(pool)
//         .await
//         .unwrap();
// }

// pub async fn move_task_order(pool: &SqlitePool, task: &Task) {
//     // TODO: We have to add some kind of ordering mechanism to tasks
//     sqlx::query!("update task set column_id = ?1", 1)
//         .execute(pool)
//         .await
//         .unwrap();
// }

// pub async fn delete_task(pool: &SqlitePool, task: &Task) {
//     // TODO: We have to add ids to tasks
//     sqlx::query!("delete from task where id = ?1", 1)
//         .execute(pool)
//         .await
//         .unwrap();
// }

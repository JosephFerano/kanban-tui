use crate::{Column, Task};
use rusqlite::{params, Connection, Result};

pub fn get_tasks_by_column(conn: &Connection, column_name: &String) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        r#"
            select task.id, title, description from task
            join kb_column on column_id = kb_column.id
            where kb_column.name = ?1
            order by sort_order
        "#,
    )?;
    let mut tasks = Vec::new();
    let rows = stmt.query_map(&[column_name], |row| {
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

pub fn get_all_columns(conn: &Connection) -> Result<Vec<Column>> {
    let mut stmt = conn.prepare("select id, name from kb_column")?;
    let query_rows = stmt.query_map((), |row| {
        Ok((row.get::<usize, i64>(0)?, row.get::<usize, String>(1)?))
    })?;
    let mut columns: Vec<Column> = Vec::new();
    for row in query_rows {
        let r = &row?;
        let name = &r.1;
        let tasks = get_tasks_by_column(conn, &name).unwrap();
        let col = Column {
            id: r.0,
            name: name.clone(),
            tasks: tasks,
            selected_task_idx: 0,
        };
        columns.push(col);
    }
    Ok(columns)
}

pub fn insert_new_task(
    conn: &Connection,
    title: String,
    description: String,
    column: &Column,
) -> Task {
    let mut stmt = conn
        .prepare("insert into task(title, description, column_id) values (?1, ?2, ?3)")
        .unwrap();
    stmt.execute(params![title, description, column.id])
        .unwrap();
    let id = conn.last_insert_rowid();
    Task {
        id,
        title,
        description,
    }
}

pub fn delete_task(conn: &Connection, task: &Task) {
    let mut stmt = conn.prepare("delete from task where id = ?1").unwrap();
    stmt.execute([task.id]).unwrap();
}

pub fn update_task_text(conn: &Connection, task: &Task) {
    let mut stmt = conn
        .prepare("update task set title = ?2, description = ?3 where id = ?1")
        .unwrap();
    stmt.execute((&task.id, &task.title, &task.description))
        .unwrap();
}

pub fn move_task_to_column(conn: &Connection, task: &Task, target_column: &Column) {
    let mut stmt = conn
        .prepare("update task set column_id = ?2, sort_order = ?3 where task.id = ?1")
        .unwrap();
    stmt.execute((&task.id, &target_column.id, &target_column.tasks.len()))
        .unwrap();
}

pub fn swap_task_order(conn: &mut Connection, task1: &Task, task2: &Task) {
    let tx = conn.transaction().unwrap();

    tx.execute(
        "create temp table temp_order as select sort_order from task where id = ?1",
        &[&task1.id],
    )
    .unwrap();
    tx.execute(
        "update task set sort_order = (select sort_order from task where id = ?2)
         where id = ?1",
        (task1.id, task2.id),
    )
    .unwrap();
    tx.execute(
        "update task set sort_order = (select sort_order from temp_order) where id = ?1",
        &[&task2.id],
    )
    .unwrap();
    tx.execute("drop table temp_order", ()).unwrap();

    tx.commit().unwrap();
}

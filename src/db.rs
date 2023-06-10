use crate::app::Task;
use sqlx::SqlitePool;

pub async fn get_tasks_by_column(pool: &SqlitePool, column_name: &String) -> Option<Vec<Task>> {
    sqlx::query_as!(
        Task,
        r#"
            select title, description from task
            join kb_column on column_id = kb_column.id
            where kb_column.name = ?1
        "#,
        column_name
    )
    .fetch_all(pool)
    .await
    .ok()
}

pub async fn get_all_tasks(pool: &SqlitePool) -> Option<Vec<(String, Vec<Task>)>> {
    let columns = sqlx::query!("select name from kb_column")
        .fetch_all(pool)
        .await
        .unwrap();
    let mut tasks_by_column: Vec<(String, Vec<Task>)> = Vec::with_capacity(columns.len());
    for col in columns {
        let tasks = get_tasks_by_column(pool, &col.name).await.unwrap();
        tasks_by_column.push((col.name, tasks));
    }
    Some(tasks_by_column)
}

use sqlx::SqlitePool;
use crate::app::Task;

pub async fn get_all_tasks(pool: SqlitePool) -> Option<Vec<Task>> {
    sqlx::query_as!(Task,
        r#"
            select title, description from task
        "#
    )
    .fetch_all(&pool)
    .await
    .ok()
}

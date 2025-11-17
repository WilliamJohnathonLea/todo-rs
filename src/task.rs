use iced::futures::TryFutureExt;
use sqlx::{Pool, Sqlite};

#[derive(Clone, Debug, Default)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub lane: String,
}

#[derive(Clone, Debug, Default)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
    pub lane: String,
    pub in_backlog: bool,
    pub project_id: i64,
}

#[derive(Clone, Debug)]
pub enum Modal {
    NewTask,
    ViewTask(i64),
    EditTask(i64),
}

impl NewTask {
    pub fn new(
        title: String,
        description: Option<String>,
        lane: String,
        in_backlog: bool,
        project_id: i64,
    ) -> Self {
        NewTask {
            title,
            description,
            lane,
            in_backlog,
            project_id,
        }
    }
}

pub async fn get_backlog_tasks(pool: Pool<Sqlite>, project_id: i64) -> Result<Vec<Task>, String> {
    sqlx::query_as!(
        Task,
        "SELECT id, title, description, lane FROM tasks WHERE in_backlog AND project_id = ?",
        project_id
    )
    .fetch_all(&pool)
    .map_err(|err| format!("got db err: {err}"))
    .await
}

pub async fn get_sprint_tasks(pool: Pool<Sqlite>, project_id: i64) -> Result<Vec<Task>, String> {
    sqlx::query_as!(
        Task,
        "SELECT id, title, description, lane FROM tasks WHERE NOT in_backlog AND project_id = ?",
        project_id
    )
    .fetch_all(&pool)
    .map_err(|err| format!("got db err: {err}"))
    .await
}

pub async fn insert_task(pool: Pool<Sqlite>, t: NewTask) -> Result<(), String> {
    sqlx::query!(
        "INSERT INTO tasks (title, description, lane, in_backlog, project_id) VALUES (?, ?, ?, ?, ?)",
        t.title,
        t.description,
        t.lane,
        t.in_backlog,
        t.project_id,
    )
    .execute(&pool)
    .map_err(|_| "Error inserting task into db".into())
    .map_ok(|_| println!("Inserted task with title {} in project {}", t.title, t.project_id))
    .await
}

pub async fn remove_task(pool: Pool<Sqlite>, task_id: i64) -> Result<(), String> {
    sqlx::query!("DELETE FROM tasks WHERE id = ?", task_id)
        .execute(&pool)
        .map_err(|_| "Error deleting task from db".into())
        .map_ok(|_| ())
        .await
}

pub async fn edit_task(pool: Pool<Sqlite>, task: Task) -> Result<(), String> {
    sqlx::query!(
        "UPDATE tasks SET title = ?, description = ?, lane = ? WHERE id = ?",
        task.title,
        task.description,
        task.lane,
        task.id
    )
    .execute(&pool)
    .map_err(|_| "Error editing task in db".into())
    .map_ok(|_| ())
    .await
}

pub async fn move_to_backlog(pool: Pool<Sqlite>, task_id: i64) -> Result<(), String> {
    sqlx::query!(
        "UPDATE tasks SET in_backlog = ? WHERE id = ?",
        true,
        task_id
    )
    .execute(&pool)
    .map_err(|_| "Error editing task in db".into())
    .map_ok(|_| ())
    .await
}

pub async fn move_to_sprint(pool: Pool<Sqlite>, task_id: i64) -> Result<(), String> {
    sqlx::query!(
        "UPDATE tasks SET in_backlog = ? WHERE id = ?",
        false,
        task_id
    )
    .execute(&pool)
    .map_err(|_| "Error editing task in db".into())
    .map_ok(|_| ())
    .await
}

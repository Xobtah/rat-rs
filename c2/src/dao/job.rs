use crate::entities::{Job, JobStatus};
use crate::utils::db_model_to_entity_o;
use anyhow::Result;
use std::str::FromStr;

#[derive(sqlx::FromRow, Clone, Debug)]
pub struct DbJob {
    pub id: i32,
    pub agent_id: Option<i32>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub name: String,
    pub task: String,
    pub result: Option<String>,
}

impl TryInto<Job> for DbJob {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Job, Self::Error> {
        Ok(Job {
            id: self.id,
            agent_id: self.agent_id,
            issued_at: self.issued_at,
            status: JobStatus::from_str(&self.status)?,
            completed_at: self.completed_at,
            name: self.name,
            task: self.task,
            result: self.result,
        })
    }
}

pub async fn init(db_pool: &sqlx::SqlitePool) -> Result<sqlx::sqlite::SqliteQueryResult> {
    Ok(sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS JOBS (
    id INTEGER PRIMARY KEY NOT NULL,
    agent_id INTEGER,
    issued_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'Issued',
    completed_at TIMESTAMP,
    name TEXT NOT NULL,
    task TEXT NOT NULL,
    result TEXT,
    FOREIGN KEY (agent_id) REFERENCES AGENTS (id)
);"#,
    )
    .execute(db_pool)
    .await?)
}

pub async fn get_next(db: &sqlx::SqlitePool, agent_id: i32) -> Result<Option<Job>> {
    sqlx::query_as::<_, DbJob>(
        r#"SELECT * FROM JOBS
        WHERE agent_id = ? AND status = 'Issued'
        ORDER BY issued_at ASC LIMIT 1"#,
    )
    .bind(agent_id)
    .fetch_optional(db)
    .await
    .map(db_model_to_entity_o)?
}

pub async fn update_running(db: &sqlx::SqlitePool, agent_id: i32, id: i32) -> Result<Option<Job>> {
    sqlx::query_as::<_, DbJob>(
        r#"UPDATE JOBS
SET status = 'Running'
WHERE id = ? AND agent_id = ?
RETURNING *"#,
    )
    .bind(id)
    .bind(agent_id)
    .fetch_optional(db)
    .await
    .map(db_model_to_entity_o)?
}

pub async fn update_result(
    db: &sqlx::SqlitePool,
    agent_id: i32,
    id: i32,
    completed_at: chrono::DateTime<chrono::Utc>,
    result: Result<String, String>,
) -> Result<Option<Job>> {
    let (stdout, stderr) = match result {
        Ok(stdout) => (Some(stdout), None),
        Err(stderr) => (None, Some(stderr)),
    };
    sqlx::query_as::<_, DbJob>(
        r#"UPDATE JOBS
SET completed_at = ?,
    status = ?,
    result = ?
    WHERE id = ? AND agent_id = ?
    RETURNING *"#,
    )
    .bind(completed_at.timestamp())
    .bind(
        if stdout.is_some() {
            JobStatus::Succeeded
        } else {
            JobStatus::Failed
        }
        .to_string(),
    )
    .bind(stdout.or(stderr))
    .bind(id)
    .bind(agent_id)
    .fetch_optional(db)
    .await
    .map(db_model_to_entity_o)?
}

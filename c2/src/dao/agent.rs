use crate::entities::Agent;
use crate::utils;
use anyhow::Result;
use common::UserAgent;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(sqlx::FromRow, Serialize, Deserialize, Default)]
pub struct DbAgent {
    pub id: i32,
    pub mac_address: String,
    pub user_agent: String,
    pub sha256: String,
    pub name: String,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

impl From<Agent> for DbAgent {
    fn from(agent: Agent) -> Self {
        Self {
            id: agent.id,
            mac_address: agent.mac_address,
            user_agent: agent.user_agent.to_string(),
            sha256: agent.sha256,
            name: agent.name,
            registered_at: agent.registered_at,
            last_seen_at: agent.last_seen_at,
        }
    }
}

impl TryInto<Agent> for DbAgent {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Agent> {
        Ok(Agent {
            id: self.id,
            mac_address: self.mac_address,
            user_agent: UserAgent::from_str(&self.user_agent)?,
            sha256: self.sha256,
            name: self.name,
            registered_at: self.registered_at,
            last_seen_at: self.last_seen_at,
        })
    }
}

pub async fn init(db_pool: &sqlx::SqlitePool) -> Result<sqlx::sqlite::SqliteQueryResult> {
    Ok(sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS AGENTS (
    id INTEGER PRIMARY KEY NOT NULL,
    mac_address TEXT UNIQUE NOT NULL,
    user_agent TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    name TEXT,
    registered_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);"#,
    )
    .execute(db_pool)
    .await?)
}

pub async fn merge(db: &sqlx::SqlitePool, agent: Agent) -> Result<Agent> {
    let sha = agent.user_agent.hash.clone();
    let agent = DbAgent::from(agent);
    sqlx::query_as::<_, DbAgent>(
        r#"INSERT INTO AGENTS (mac_address, user_agent, sha256, name, last_seen_at)
VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
ON CONFLICT (mac_address) DO UPDATE SET user_agent = $2, sha256= $3, name = $4, last_seen_at = CURRENT_TIMESTAMP
RETURNING *"#,
    )
    .bind(agent.mac_address)
    .bind(agent.user_agent)
    .bind(if agent.sha256.is_empty() { sha } else { agent.sha256 })
    .bind(agent.name.trim())
    .fetch_one(db)
    .await
    .map(DbAgent::try_into)?
}

pub async fn get_by_id(db: &sqlx::SqlitePool, id: i32) -> Result<Option<Agent>> {
    sqlx::query_as::<_, DbAgent>("SELECT * FROM AGENTS WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .map(utils::db_model_to_entity_o::<Agent, DbAgent>)?
}

pub async fn update_last_seen_at(db: &sqlx::SqlitePool, agent: &Agent) -> Result<()> {
    sqlx::query(
        r#"UPDATE AGENTS
SET
    last_seen_at = CURRENT_TIMESTAMP,
    user_agent = $2,
    sha256 = $3
WHERE id = $1"#,
    )
    .bind(agent.id)
    .bind(agent.user_agent.to_string())
    .bind(agent.sha256.clone())
    .execute(db)
    .await?;
    Ok(())
}

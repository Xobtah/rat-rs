pub mod agent;
pub mod bin;
pub mod job;

use crate::entities::Job;
use anyhow::Result;
use common::Message;
use log::info;
use sqlx::migrate::MigrateDatabase;

impl TryInto<Message> for Job {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<Message, Self::Error> {
        Ok(Message::Job {
            id: self.id,
            task: serde_json::from_str(&self.task)?,
        })
    }
}

pub async fn init() -> Result<sqlx::Pool<sqlx::Sqlite>> {
    const DB_URL: &str = "sqlite://db/sqlite.db";
    if !sqlx::Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        info!("Creating database {}", DB_URL);
        match sqlx::Sqlite::create_database(DB_URL).await {
            Ok(_) => info!("Create db success"),
            Err(error) => panic!("Database creation error: {}", error),
        }
    } else {
        info!("Database already exists");
    }
    let db = sqlx::SqlitePool::connect(DB_URL).await.unwrap();
    agent::init(&db).await?;
    job::init(&db).await?;
    bin::init(&db).await?;
    Ok(db)
}

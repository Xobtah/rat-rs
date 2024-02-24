use anyhow::Result;

#[derive(sqlx::FromRow, Clone, Debug)]
pub struct DbBin {
    pub id: i32,
    pub sha256: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub bytes: Vec<u8>,
}

pub async fn init(db_pool: &sqlx::SqlitePool) -> Result<sqlx::sqlite::SqliteQueryResult> {
    Ok(sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS BINS (
    id INTEGER PRIMARY KEY NOT NULL,
    sha256 TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    bytes BLOB
);"#,
    )
    .execute(db_pool)
    .await?)
}

pub async fn get_bin_by_sha256(db: &sqlx::SqlitePool, sha256: &str) -> Result<Option<Vec<u8>>> {
    Ok(sqlx::query_as::<_, DbBin>(
        r#"SELECT * FROM BINS
        WHERE sha256 = ?"#,
    )
    .bind(sha256)
    .fetch_optional(db)
    .await
    .map(|agent_bin_o| agent_bin_o.map(|agent_bin| agent_bin.bytes))?)
}
